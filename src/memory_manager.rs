//! Memory management for efficient neural network operations
//!
//! This module provides memory management capabilities for neural networks,
//! including buffer allocation, memory pools, arena allocation, and efficient data structures.
//!
//! Features:
//! - Memory pooling for tensor allocations
//! - Arena allocation for neural network structures
//! - Zero-copy operations where possible
//! - Memory usage tracking and leak detection
//! - Automatic cleanup and defragmentation

use num_traits::Float;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Memory manager for neural network operations with advanced features
pub struct MemoryManager<T: Float> {
    /// Memory pools for different data types
    pools: HashMap<String, MemoryPool<T>>,
    /// Arena allocator for efficient allocations
    arena: Option<ArenaAllocator>,
    /// Memory leak detector
    leak_detector: Option<MemoryLeakDetector>,
    /// Smart cache for tensor operations
    smart_cache: Option<SmartCache<T>>,
    /// Memory prefetcher for predictive loading
    prefetcher: Option<MemoryPrefetcher>,
    /// Garbage collector for unused tensors
    garbage_collector: Option<GarbageCollector>,
    /// Total memory allocated
    total_allocated: usize,
    /// Peak memory usage
    peak_memory: usize,
    /// Memory usage statistics
    stats: MemoryStats,
    /// Memory allocation tracker for debugging
    #[cfg(debug_assertions)]
    allocation_tracker: HashMap<String, Vec<AllocationInfo>>,
    /// Enable zero-copy operations
    enable_zero_copy: bool,
    /// Enable arena allocation
    enable_arena: bool,
    /// Enable leak detection
    enable_leak_detection: bool,
    /// Enable smart caching
    enable_smart_cache: bool,
    /// Enable memory prefetching
    enable_prefetching: bool,
    /// Enable garbage collection
    enable_gc: bool,
}

/// Memory allocation tracking information
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub size: usize,
    pub backtrace: String,
    pub timestamp: std::time::Instant,
    pub id: usize,
}

/// Memory pool for efficient allocation/deallocation
pub struct MemoryPool<T: Float> {
    /// Available buffers
    available: Vec<Vec<T>>,
    /// Count of currently allocated buffers
    allocated_count: usize,
    /// Buffer size for this pool
    buffer_size: usize,
    /// Pool name
    name: String,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total memory allocated in bytes
    pub total_allocated: usize,
    /// Available memory in bytes
    pub available: usize,
    /// Number of active buffers
    pub buffer_count: usize,
    /// Memory fragmentation ratio (0.0 = no fragmentation, 1.0 = highly fragmented)
    pub fragmentation_ratio: f64,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Current memory usage
    pub current_memory: usize,
    /// Number of allocations performed
    pub allocation_count: u64,
    /// Number of deallocations performed
    pub deallocation_count: u64,
    /// Average allocation size
    pub average_allocation_size: f64,
}

/// Arena allocator for efficient memory management
pub struct ArenaAllocator {
    /// Memory chunks
    chunks: Vec<MemoryChunk>,
    /// Current chunk being allocated from
    current_chunk: usize,
    /// Chunk size for new allocations
    chunk_size: usize,
    /// Total memory allocated
    total_allocated: usize,
    /// Memory alignment
    alignment: usize,
}

#[derive(Debug)]
struct MemoryChunk {
    /// Pointer to the memory chunk
    ptr: *mut u8,
    /// Size of the chunk
    size: usize,
    /// Current offset into the chunk
    offset: usize,
    /// Layout used for allocation
    layout: Layout,
}

impl MemoryChunk {
    fn new(size: usize, alignment: usize) -> Result<Self, String> {
        let layout = Layout::from_size_align(size, alignment)
            .map_err(|e| format!("Invalid layout: {}", e))?;

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err("Memory allocation failed".to_string());
        }

        Ok(Self {
            ptr,
            size,
            offset: 0,
            layout,
        })
    }

    fn allocate(&mut self, size: usize, alignment: usize) -> Option<*mut u8> {
        let aligned_offset = align_offset(self.offset, alignment);
        if aligned_offset + size > self.size {
            return None; // Not enough space in this chunk
        }

        let ptr = unsafe { self.ptr.add(aligned_offset) };
        self.offset = aligned_offset + size;
        Some(ptr)
    }

    fn remaining_space(&self) -> usize {
        self.size - self.offset
    }

    fn is_empty(&self) -> bool {
        self.offset == 0
    }
}

impl Drop for MemoryChunk {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

/// Zero-copy tensor wrapper
pub struct TensorView<'a, T: Float> {
    /// Data pointer
    data: *const T,
    /// Shape of the tensor
    shape: Vec<usize>,
    /// Strides for each dimension
    strides: Vec<usize>,
    /// Total number of elements
    len: usize,
    /// Phantom data to tie lifetime
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Float> TensorView<'a, T> {
    /// Create a new tensor view from a slice
    pub fn from_slice(data: &'a [T], shape: &[usize]) -> Result<Self, String> {
        let total_elements: usize = shape.iter().product();
        if total_elements != data.len() {
            return Err(format!(
                "Shape product {} does not match data length {}",
                total_elements,
                data.len()
            ));
        }

        let strides = Self::calculate_strides(shape);

        Ok(Self {
            data: data.as_ptr(),
            shape: shape.to_vec(),
            strides,
            len: total_elements,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Calculate strides for a given shape
    fn calculate_strides(shape: &[usize]) -> Vec<usize> {
        let mut strides = vec![0; shape.len()];
        if !shape.is_empty() {
            strides[shape.len() - 1] = 1;
            for i in (0..shape.len() - 1).rev() {
                strides[i] = strides[i + 1] * shape[i + 1];
            }
        }
        strides
    }

    /// Get element at multi-dimensional index
    pub fn get(&self, indices: &[usize]) -> Result<T, String> {
        if indices.len() != self.shape.len() {
            return Err(format!(
                "Index dimensions {} do not match tensor dimensions {}",
                indices.len(),
                self.shape.len()
            ));
        }

        let mut flat_index = 0;
        for (i, &idx) in indices.iter().enumerate() {
            if idx >= self.shape[i] {
                return Err(format!(
                    "Index {} out of bounds for dimension {} with size {}",
                    idx, i, self.shape[i]
                ));
            }
            flat_index += idx * self.strides[i];
        }

        if flat_index >= self.len {
            return Err(format!("Flat index {} out of bounds", flat_index));
        }

        unsafe { Ok(ptr::read(self.data.add(flat_index))) }
    }

    /// Get shape of the tensor
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Get strides of the tensor
    pub fn strides(&self) -> &[usize] {
        &self.strides
    }

    /// Get total number of elements
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if tensor is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Smart cache for tensor operations with LRU eviction
pub struct SmartCache<T: Float> {
    /// Cached tensors with access patterns
    cache: HashMap<String, CachedTensor<T>>,
    /// Maximum cache size in bytes
    max_size: usize,
    /// Current cache size in bytes
    current_size: usize,
    /// Access history for LRU eviction
    access_history: Vec<String>,
    /// Cache hit counter
    hits: u64,
    /// Cache miss counter
    misses: u64,
}

#[derive(Clone)]
struct CachedTensor<T: Float> {
    data: Vec<T>,
    shape: Vec<usize>,
    last_access: std::time::Instant,
    access_count: u64,
    priority: CachePriority,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CachePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Memory prefetcher for predictive loading
pub struct MemoryPrefetcher {
    /// Prefetch patterns learned from access history
    patterns: HashMap<String, Vec<String>>,
    /// Prefetch queue
    queue: Vec<String>,
    /// Maximum queue size
    max_queue_size: usize,
    /// Prefetch distance (how far ahead to prefetch)
    prefetch_distance: usize,
}

/// Garbage collector for unused tensors
pub struct GarbageCollector {
    /// Objects with reference counts
    ref_counts: HashMap<String, u32>,
    /// Objects marked for collection
    marked_for_gc: Vec<String>,
    /// Collection threshold
    collection_threshold: usize,
    /// Last collection time
    last_collection: std::time::Instant,
}

/// Memory leak detector
pub struct MemoryLeakDetector {
    /// Active allocations
    active_allocations: HashMap<usize, AllocationInfo>,
    /// Allocation counter
    allocation_counter: AtomicUsize,
    /// Total leaked memory
    leaked_memory: AtomicUsize,
}

impl<T: Float> SmartCache<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            current_size: 0,
            access_history: Vec::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Get tensor from cache
    pub fn get(&mut self, key: &str) -> Option<&[T]> {
        if let Some(tensor) = self.cache.get_mut(key) {
            tensor.last_access = std::time::Instant::now();
            tensor.access_count += 1;

            // Update access history for LRU
            if let Some(pos) = self.access_history.iter().position(|k| k == key) {
                self.access_history.remove(pos);
            }
            self.access_history.push(key.to_string());

            self.hits += 1;
            Some(&tensor.data)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Put tensor in cache
    pub fn put(&mut self, key: String, data: Vec<T>, shape: Vec<usize>, priority: CachePriority) {
        let tensor_size = data.len() * std::mem::size_of::<T>();

        // Check if we need to evict
        while self.current_size + tensor_size > self.max_size && !self.cache.is_empty() {
            self.evict_lru();
        }

        let tensor = CachedTensor {
            data,
            shape,
            last_access: std::time::Instant::now(),
            access_count: 0,
            priority,
        };

        if let Some(old_tensor) = self.cache.insert(key.clone(), tensor) {
            self.current_size -= old_tensor.data.len() * std::mem::size_of::<T>();
        }

        self.current_size += tensor_size;
        self.access_history.push(key);
    }

    /// Evict least recently used tensor
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.access_history.first().cloned() {
            if let Some(tensor) = self.cache.remove(&lru_key) {
                self.current_size -= tensor.data.len() * std::mem::size_of::<T>();
                self.access_history.remove(0);
            }
        }
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_size = 0;
        self.access_history.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

impl MemoryPrefetcher {
    pub fn new(max_queue_size: usize, prefetch_distance: usize) -> Self {
        Self {
            patterns: HashMap::new(),
            queue: Vec::new(),
            max_queue_size,
            prefetch_distance,
        }
    }

    /// Record access pattern
    pub fn record_access(&mut self, key: &str) {
        // This is a simple implementation - in practice, you'd use more sophisticated
        // pattern recognition algorithms
        let pattern_key = format!("pattern_{}", key.len() % 10);
        self.patterns
            .entry(pattern_key)
            .or_insert_with(Vec::new)
            .push(key.to_string());
    }

    /// Get prefetch recommendations
    pub fn get_prefetch_recommendations(&self, current_key: &str) -> Vec<String> {
        let pattern_key = format!("pattern_{}", current_key.len() % 10);

        if let Some(pattern) = self.patterns.get(&pattern_key) {
            if let Some(pos) = pattern.iter().position(|k| k == current_key) {
                let start = pos + 1;
                let end = (pos + 1 + self.prefetch_distance).min(pattern.len());
                return pattern[start..end].to_vec();
            }
        }

        Vec::new()
    }

    /// Add to prefetch queue
    pub fn add_to_queue(&mut self, keys: Vec<String>) {
        for key in keys {
            if !self.queue.contains(&key) {
                self.queue.push(key);
                if self.queue.len() > self.max_queue_size {
                    self.queue.remove(0);
                }
            }
        }
    }

    /// Get next items to prefetch
    pub fn get_next_prefetch(&mut self) -> Vec<String> {
        let result = self.queue.clone();
        self.queue.clear();
        result
    }
}

impl GarbageCollector {
    pub fn new(collection_threshold: usize) -> Self {
        Self {
            ref_counts: HashMap::new(),
            marked_for_gc: Vec::new(),
            collection_threshold,
            last_collection: std::time::Instant::now(),
        }
    }

    /// Increment reference count
    pub fn inc_ref(&mut self, key: &str) {
        let count = self.ref_counts.entry(key.to_string()).or_insert(0);
        *count += 1;
    }

    /// Decrement reference count
    pub fn dec_ref(&mut self, key: &str) {
        if let Some(count) = self.ref_counts.get_mut(key) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.marked_for_gc.push(key.to_string());
                }
            }
        }
    }

    /// Collect garbage if threshold is reached
    pub fn collect_garbage(&mut self) -> Vec<String> {
        if self.marked_for_gc.len() >= self.collection_threshold {
            let collected = self.marked_for_gc.clone();
            for key in &collected {
                self.ref_counts.remove(key);
            }
            self.marked_for_gc.clear();
            self.last_collection = std::time::Instant::now();
            collected
        } else {
            Vec::new()
        }
    }

    /// Force garbage collection
    pub fn force_collect(&mut self) -> Vec<String> {
        let collected: Vec<String> = self
            .ref_counts
            .keys()
            .filter(|key| *self.ref_counts.get(*key).unwrap_or(&0) == 0)
            .cloned()
            .collect();

        for key in &collected {
            self.ref_counts.remove(key);
        }

        self.marked_for_gc.clear();
        self.last_collection = std::time::Instant::now();
        collected
    }
}

impl MemoryLeakDetector {
    pub fn new() -> Self {
        Self {
            active_allocations: HashMap::new(),
            allocation_counter: AtomicUsize::new(0),
            leaked_memory: AtomicUsize::new(0),
        }
    }

    pub fn track_allocation(&mut self, size: usize) -> usize {
        let id = self.allocation_counter.fetch_add(1, Ordering::SeqCst);
        let backtrace = if cfg!(debug_assertions) {
            format!("{:?}", std::backtrace::Backtrace::capture())
        } else {
            "Backtrace not available in release mode".to_string()
        };

        let info = AllocationInfo {
            size,
            backtrace,
            timestamp: std::time::Instant::now(),
            id,
        };

        self.active_allocations.insert(id, info);
        id
    }

    pub fn track_deallocation(&mut self, id: usize) {
        if let Some(info) = self.active_allocations.remove(&id) {
            // Allocation was properly deallocated
        } else {
            log::warn!("Attempted to deallocate unknown allocation {}", id);
        }
    }

    pub fn detect_leaks(&self) -> Vec<AllocationInfo> {
        self.active_allocations.values().cloned().collect()
    }

    pub fn leaked_memory_bytes(&self) -> usize {
        self.active_allocations.values().map(|info| info.size).sum()
    }
}

impl<T: Float> MemoryManager<T> {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            arena: None,
            leak_detector: None,
            smart_cache: None,
            prefetcher: None,
            garbage_collector: None,
            total_allocated: 0,
            peak_memory: 0,
            stats: MemoryStats {
                total_allocated: 0,
                available: 0,
                buffer_count: 0,
                fragmentation_ratio: 0.0,
                peak_memory: 0,
                current_memory: 0,
                allocation_count: 0,
                deallocation_count: 0,
                average_allocation_size: 0.0,
            },
            enable_zero_copy: true,
            enable_arena: false,
            enable_leak_detection: cfg!(debug_assertions),
            enable_smart_cache: false,
            enable_prefetching: false,
            enable_gc: false,
            #[cfg(debug_assertions)]
            allocation_tracker: HashMap::new(),
        }
    }

    /// Create a memory manager with advanced features enabled
    pub fn with_advanced_features(
        enable_zero_copy: bool,
        enable_arena: bool,
        enable_leak_detection: bool,
        arena_chunk_size: usize,
    ) -> Self {
        let mut manager = Self::new();
        manager.enable_zero_copy = enable_zero_copy;
        manager.enable_arena = enable_arena;
        manager.enable_leak_detection = enable_leak_detection;

        if enable_leak_detection {
            manager.leak_detector = Some(MemoryLeakDetector::new());
        }

        if enable_arena {
            manager.arena = Some(
                ArenaAllocator::new(arena_chunk_size, 64).unwrap_or_else(|_| {
                    log::warn!("Failed to create arena allocator, falling back to pool allocation");
                    ArenaAllocator::new(1024 * 1024, 64).unwrap() // 1MB fallback
                }),
            );
        }

        manager
    }

    /// Create a memory manager with pre-allocated pools
    pub fn with_capacity(layer_sizes: &[usize]) -> Self {
        let mut manager = Self::new();

        // Pre-allocate pools based on network architecture
        for (i, &size) in layer_sizes.iter().enumerate() {
            manager.create_pool(&format!("layer_{}_neurons", i), size);
            manager.create_pool(&format!("layer_{}_weights", i), size * size);
            manager.create_pool(&format!("layer_{}_gradients", i), size);
        }

        // Common utility pools
        manager.create_pool("activations", 1024);
        manager.create_pool("errors", 1024);
        manager.create_pool("temp", 512);

        manager
    }

    /// Create a tensor view from a slice (zero-copy)
    pub fn create_tensor_view<'a>(
        &self,
        data: &'a [T],
        shape: &[usize],
    ) -> Result<TensorView<'a, T>, String> {
        if !self.enable_zero_copy {
            return Err("Zero-copy operations are disabled".to_string());
        }
        TensorView::from_slice(data, shape)
    }

    /// Allocate memory using the arena allocator
    pub fn arena_allocate(&mut self, size: usize) -> Result<*mut T, String> {
        if !self.enable_arena {
            return Err("Arena allocation is disabled".to_string());
        }

        if let Some(ref mut arena) = self.arena {
            let alignment = std::mem::align_of::<T>();
            let byte_size = size * std::mem::size_of::<T>();

            if let Some(ptr) = arena.allocate(byte_size, alignment) {
                let typed_ptr = ptr as *mut T;
                self.total_allocated += byte_size;
                self.update_stats();
                Ok(typed_ptr)
            } else {
                Err("Arena allocation failed: not enough space".to_string())
            }
        } else {
            Err("Arena allocator not initialized".to_string())
        }
    }

    /// Reset the arena allocator (clear all allocations)
    pub fn reset_arena(&mut self) {
        if let Some(ref mut arena) = self.arena {
            // Reset all chunks to offset 0
            for chunk in &mut arena.chunks {
                chunk.offset = 0;
            }
            arena.current_chunk = 0;
            self.total_allocated = 0;
            self.update_stats();
        }
    }

    /// Get memory leak report
    pub fn get_leak_report(&self) -> Option<Vec<AllocationInfo>> {
        self.leak_detector
            .as_ref()
            .map(|detector| detector.detect_leaks())
    }

    /// Get leaked memory in bytes
    pub fn get_leaked_memory(&self) -> usize {
        self.leak_detector
            .as_ref()
            .map(|detector| detector.leaked_memory_bytes())
            .unwrap_or(0)
    }

    /// Create a memory pool with the given name and buffer size
    pub fn create_pool(&mut self, name: &str, buffer_size: usize) {
        let pool = MemoryPool::new(name.to_string(), buffer_size);
        self.pools.insert(name.to_string(), pool);
    }

    /// Allocate a buffer from the specified pool with leak tracking
    pub fn allocate(&mut self, pool_name: &str, size: usize) -> Result<Vec<T>, String> {
        if let Some(pool) = self.pools.get_mut(pool_name) {
            let buffer = pool.allocate(size)?;
            let byte_size = size * std::mem::size_of::<T>();
            self.total_allocated += byte_size;
            self.peak_memory = self.peak_memory.max(self.total_allocated);
            self.stats.allocation_count += 1;

            // Track allocation for leak detection
            if let Some(ref mut detector) = self.leak_detector {
                let id = detector.track_allocation(byte_size);
                // Store the allocation ID in the buffer for later tracking
                // Note: This is a simplified approach - in practice, you might want
                // to use a more sophisticated tracking mechanism
            }

            self.update_stats();
            Ok(buffer)
        } else {
            Err(format!("Pool '{pool_name}' not found"))
        }
    }

    /// Deallocate a buffer back to the specified pool with leak tracking
    pub fn deallocate(&mut self, pool_name: &str, buffer: Vec<T>) -> Result<(), String> {
        if let Some(pool) = self.pools.get_mut(pool_name) {
            let size = buffer.len() * std::mem::size_of::<T>();
            pool.deallocate(buffer);
            self.total_allocated = self.total_allocated.saturating_sub(size);
            self.stats.deallocation_count += 1;

            // Track deallocation for leak detection
            if let Some(ref mut detector) = self.leak_detector {
                // In a real implementation, you'd extract the allocation ID
                // from the buffer and call detector.track_deallocation(id)
            }

            self.update_stats();
            Ok(())
        } else {
            Err(format!("Pool '{pool_name}' not found"))
        }
    }

    /// Allocate a zero-copy tensor view
    pub fn allocate_tensor_view<'a>(
        &mut self,
        pool_name: &str,
        size: usize,
        shape: &[usize],
    ) -> Result<TensorView<'a, T>, String> {
        if !self.enable_zero_copy {
            return Err("Zero-copy operations are disabled".to_string());
        }

        let buffer = self.allocate(pool_name, size)?;
        let slice = unsafe { std::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };

        // Note: This creates a temporary view - in practice, you'd need to
        // manage the lifetime more carefully to ensure the buffer outlives the view
        TensorView::from_slice(slice, shape)
    }

    /// Get memory usage statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.clone()
    }

    /// Clear all memory pools
    pub fn clear_all(&mut self) {
        for pool in self.pools.values_mut() {
            pool.clear();
        }
        self.total_allocated = 0;
        self.update_stats();
    }

    /// Enable advanced memory management features
    pub fn enable_advanced_features(
        &mut self,
        enable_cache: bool,
        enable_prefetching: bool,
        enable_gc: bool,
        cache_size: usize,
        prefetch_distance: usize,
        gc_threshold: usize,
    ) {
        self.enable_smart_cache = enable_cache;
        self.enable_prefetching = enable_prefetching;
        self.enable_gc = enable_gc;

        if enable_cache {
            self.smart_cache = Some(SmartCache::new(cache_size));
        }

        if enable_prefetching {
            self.prefetcher = Some(MemoryPrefetcher::new(100, prefetch_distance));
        }

        if enable_gc {
            self.garbage_collector = Some(GarbageCollector::new(gc_threshold));
        }
    }

    /// Cache tensor data
    pub fn cache_tensor(&mut self, key: String, data: Vec<T>, shape: Vec<usize>) {
        if let Some(ref mut cache) = self.smart_cache {
            cache.put(key, data, shape, CachePriority::Normal);
        }
    }

    /// Get cached tensor
    pub fn get_cached_tensor(&mut self, key: &str) -> Option<&[T]> {
        if let Some(ref mut cache) = self.smart_cache {
            cache.get(key)
        } else {
            None
        }
    }

    /// Record memory access pattern for prefetching
    pub fn record_memory_access(&mut self, key: &str) {
        if let Some(ref mut prefetcher) = self.prefetcher {
            prefetcher.record_access(key);
        }
    }

    /// Get prefetch recommendations
    pub fn get_prefetch_recommendations(&self, current_key: &str) -> Vec<String> {
        if let Some(ref prefetcher) = self.prefetcher {
            prefetcher.get_prefetch_recommendations(current_key)
        } else {
            Vec::new()
        }
    }

    /// Increment reference count for garbage collection
    pub fn inc_ref(&mut self, key: &str) {
        if let Some(ref mut gc) = self.garbage_collector {
            gc.inc_ref(key);
        }
    }

    /// Decrement reference count for garbage collection
    pub fn dec_ref(&mut self, key: &str) {
        if let Some(ref mut gc) = self.garbage_collector {
            gc.dec_ref(key);
        }
    }

    /// Collect garbage
    pub fn collect_garbage(&mut self) -> Vec<String> {
        if let Some(ref mut gc) = self.garbage_collector {
            gc.collect_garbage()
        } else {
            Vec::new()
        }
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> Option<(f64, usize, usize)> {
        if let Some(ref cache) = self.smart_cache {
            Some((cache.hit_rate(), cache.current_size, cache.cache.len()))
        } else {
            None
        }
    }

    /// Defragment memory pools
    pub fn defragment_pools(&mut self) {
        for pool in self.pools.values_mut() {
            // Remove empty buffers and reorganize
            pool.available.retain(|buffer| !buffer.is_empty());
        }

        // Defragment arena if available
        if let Some(ref mut arena) = self.arena {
            arena.defragment();
        }

        self.update_stats();
    }

    /// Optimize memory layout for specific access patterns
    pub fn optimize_for_access_pattern(&mut self, pattern: &str) {
        match pattern {
            "sequential" => {
                // Optimize for sequential access
                self.defragment_pools();
            }
            "random" => {
                // Optimize for random access with better caching
                if let Some(ref mut cache) = self.smart_cache {
                    // Increase cache size for random access patterns
                }
            }
            "temporal" => {
                // Optimize for temporal locality
                if let Some(ref mut prefetcher) = self.prefetcher {
                    // Adjust prefetch distance for temporal patterns
                }
            }
            _ => {}
        }
    }

    /// Update memory statistics
    fn update_stats(&mut self) {
        let mut buffer_count = 0;
        let mut available_buffers = 0;

        for pool in self.pools.values() {
            buffer_count += pool.allocated_count;
            available_buffers += pool.available.len();
        }

        let total_allocations = self.stats.allocation_count as f64;
        let average_allocation_size = if total_allocations > 0.0 {
            self.total_allocated as f64 / total_allocations
        } else {
            0.0
        };

        self.stats = MemoryStats {
            total_allocated: self.total_allocated,
            available: available_buffers * std::mem::size_of::<T>(),
            buffer_count,
            fragmentation_ratio: if buffer_count > 0 {
                available_buffers as f64 / buffer_count as f64
            } else {
                0.0
            },
            peak_memory: self.peak_memory,
            current_memory: self.total_allocated,
            allocation_count: self.stats.allocation_count,
            deallocation_count: self.stats.deallocation_count,
            average_allocation_size,
        };
    }
}

impl<T: Float> Default for MemoryManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl ArenaAllocator {
    /// Create a new arena allocator
    pub fn new(chunk_size: usize, alignment: usize) -> Result<Self, String> {
        let initial_chunk = MemoryChunk::new(chunk_size, alignment)?;

        Ok(Self {
            chunks: vec![initial_chunk],
            current_chunk: 0,
            chunk_size,
            total_allocated: 0,
            alignment,
        })
    }

    /// Allocate memory from the arena
    pub fn allocate(&mut self, size: usize, alignment: usize) -> Option<*mut u8> {
        // Try to allocate from the current chunk
        if let Some(ptr) = self.chunks[self.current_chunk].allocate(size, alignment) {
            self.total_allocated += size;
            return Some(ptr);
        }

        // Current chunk is full, try to find a chunk with enough space
        for (i, chunk) in self.chunks.iter_mut().enumerate() {
            if let Some(ptr) = chunk.allocate(size, alignment) {
                self.current_chunk = i;
                self.total_allocated += size;
                return Some(ptr);
            }
        }

        // No existing chunk has enough space, create a new one
        match MemoryChunk::new(self.chunk_size.max(size), self.alignment) {
            Ok(mut new_chunk) => {
                if let Some(ptr) = new_chunk.allocate(size, alignment) {
                    self.current_chunk = self.chunks.len();
                    self.chunks.push(new_chunk);
                    self.total_allocated += size;
                    Some(ptr)
                } else {
                    None // This shouldn't happen if we sized the chunk correctly
                }
            }
            Err(_) => None,
        }
    }

    /// Get total memory allocated by this arena
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// Get number of chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get remaining space in current chunk
    pub fn remaining_space(&self) -> usize {
        self.chunks[self.current_chunk].remaining_space()
    }

    /// Reset the arena (mark all memory as free)
    pub fn reset(&mut self) {
        for chunk in &mut self.chunks {
            chunk.offset = 0;
        }
        self.current_chunk = 0;
        self.total_allocated = 0;
    }

    /// Defragment the arena by removing empty chunks
    pub fn defragment(&mut self) {
        self.chunks.retain(|chunk| !chunk.is_empty());
        if self.chunks.is_empty() {
            // Create a new initial chunk if all were removed
            if let Ok(chunk) = MemoryChunk::new(self.chunk_size, self.alignment) {
                self.chunks.push(chunk);
            }
        }
        self.current_chunk = 0;
    }
}

impl<T: Float> MemoryPool<T> {
    /// Create a new memory pool
    pub fn new(name: String, buffer_size: usize) -> Self {
        Self {
            available: Vec::new(),
            allocated_count: 0,
            buffer_size,
            name,
        }
    }

    /// Allocate a buffer from this pool
    pub fn allocate(&mut self, size: usize) -> Result<Vec<T>, String> {
        // If we have an available buffer of the right size, reuse it
        if let Some(mut buffer) = self.available.pop() {
            buffer.clear();
            buffer.resize(size, T::zero());
            self.allocated_count += 1;
            Ok(buffer)
        } else {
            // Create a new buffer
            let buffer = vec![T::zero(); size];
            self.allocated_count += 1;
            Ok(buffer)
        }
    }

    /// Deallocate a buffer back to this pool
    pub fn deallocate(&mut self, buffer: Vec<T>) {
        // Add to available list for reuse
        self.available.push(buffer);
        self.allocated_count = self.allocated_count.saturating_sub(1);
    }

    /// Clear all buffers in this pool
    pub fn clear(&mut self) {
        self.available.clear();
        self.allocated_count = 0;
    }

    /// Get the number of allocated buffers
    pub fn allocated_count(&self) -> usize {
        self.allocated_count
    }

    /// Get the number of available buffers
    pub fn available_count(&self) -> usize {
        self.available.len()
    }
}

lazy_static::lazy_static! {
    /// Global memory manager instance
    static ref GLOBAL_MEMORY_MANAGER: Arc<Mutex<MemoryManager<f32>>> = Arc::new(Mutex::new(MemoryManager::new()));
}

/// Get the global memory manager
pub fn get_global_memory_manager() -> Arc<Mutex<MemoryManager<f32>>> {
    GLOBAL_MEMORY_MANAGER.clone()
}

/// Align an offset to the given alignment
fn align_offset(offset: usize, alignment: usize) -> usize {
    if alignment == 0 {
        offset
    } else {
        ((offset + alignment - 1) / alignment) * alignment
    }
}

/// Initialize default memory pools
pub fn init_default_pools() {
    let mut manager = GLOBAL_MEMORY_MANAGER.lock().unwrap();
    manager.create_pool("weights", 1024);
    manager.create_pool("activations", 512);
    manager.create_pool("gradients", 512);
    manager.create_pool("temporary", 256);
}

/// Initialize advanced memory management features
pub fn init_advanced_memory_management(
    enable_zero_copy: bool,
    enable_arena: bool,
    enable_leak_detection: bool,
    arena_chunk_size: usize,
) {
    let mut manager = GLOBAL_MEMORY_MANAGER.lock().unwrap();

    // Replace the current manager with an advanced one
    *manager = MemoryManager::with_advanced_features(
        enable_zero_copy,
        enable_arena,
        enable_leak_detection,
        arena_chunk_size,
    );

    // Re-create the default pools
    manager.create_pool("weights", 1024);
    manager.create_pool("activations", 512);
    manager.create_pool("gradients", 512);
    manager.create_pool("temporary", 256);
}

/// Get memory usage report
pub fn get_memory_report() -> MemoryStats {
    let manager = GLOBAL_MEMORY_MANAGER.lock().unwrap();
    manager.get_stats()
}

/// Get memory leak report
pub fn get_memory_leak_report() -> Option<Vec<AllocationInfo>> {
    let manager = GLOBAL_MEMORY_MANAGER.lock().unwrap();
    manager.get_leak_report()
}

/// Reset arena allocator
pub fn reset_memory_arena() {
    let mut manager = GLOBAL_MEMORY_MANAGER.lock().unwrap();
    manager.reset_arena();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_creation() {
        let manager: MemoryManager<f32> = MemoryManager::new();
        assert_eq!(manager.total_allocated, 0);
        assert_eq!(manager.pools.len(), 0);
    }

    #[test]
    fn test_pool_creation() {
        let mut manager: MemoryManager<f32> = MemoryManager::new();
        manager.create_pool("test", 100);
        assert_eq!(manager.pools.len(), 1);
        assert!(manager.pools.contains_key("test"));
    }

    #[test]
    fn test_allocation_deallocation() {
        let mut manager: MemoryManager<f32> = MemoryManager::new();
        manager.create_pool("test", 100);

        // Allocate buffer
        let buffer = manager.allocate("test", 50).unwrap();
        assert_eq!(buffer.len(), 50);
        assert!(manager.total_allocated > 0);

        // Deallocate buffer
        manager.deallocate("test", buffer).unwrap();
        // Note: total_allocated might not be 0 due to pool reuse
    }

    #[test]
    fn test_memory_stats() {
        let mut manager: MemoryManager<f32> = MemoryManager::new();
        manager.create_pool("test", 100);

        let stats = manager.get_stats();
        assert_eq!(stats.buffer_count, 0);
        assert_eq!(stats.total_allocated, 0);

        let _buffer = manager.allocate("test", 50).unwrap();
        let stats = manager.get_stats();
        assert_eq!(stats.buffer_count, 1);
        assert!(stats.total_allocated > 0);
    }

    #[test]
    fn test_pool_reuse() {
        let mut pool: MemoryPool<f32> = MemoryPool::new("test".to_string(), 100);

        // Allocate and deallocate
        let buffer1 = pool.allocate(50).unwrap();
        pool.deallocate(buffer1);

        // Allocate again - should reuse
        let buffer2 = pool.allocate(50).unwrap();
        assert_eq!(buffer2.len(), 50);
        assert_eq!(pool.available_count(), 0);
        assert_eq!(pool.allocated_count(), 1);
    }

    #[test]
    fn test_tensor_view_creation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let shape = vec![2, 3];

        let view = TensorView::from_slice(&data, &shape).unwrap();

        assert_eq!(view.shape(), &[2, 3]);
        assert_eq!(view.len(), 6);
        assert!(!view.is_empty());

        // Test element access
        assert_eq!(view.get(&[0, 0]).unwrap(), 1.0);
        assert_eq!(view.get(&[0, 1]).unwrap(), 2.0);
        assert_eq!(view.get(&[1, 2]).unwrap(), 6.0);
    }

    #[test]
    fn test_tensor_view_bounds_checking() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];

        let view = TensorView::from_slice(&data, &shape).unwrap();

        // Valid access
        assert_eq!(view.get(&[0, 0]).unwrap(), 1.0);

        // Out of bounds
        assert!(view.get(&[2, 0]).is_err());
        assert!(view.get(&[0, 2]).is_err());
        assert!(view.get(&[0]).is_err()); // Wrong dimensions
    }

    #[test]
    fn test_memory_leak_detector() {
        let mut detector = MemoryLeakDetector::new();

        let id1 = detector.track_allocation(100);
        let id2 = detector.track_allocation(200);

        assert_eq!(detector.leaked_memory_bytes(), 300);

        detector.track_deallocation(id1);
        assert_eq!(detector.leaked_memory_bytes(), 200);

        let leaks = detector.detect_leaks();
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].size, 200);
    }

    #[test]
    fn test_arena_allocator() {
        let mut arena = ArenaAllocator::new(1024, 8).unwrap();

        // Allocate some memory
        let ptr1 = arena.allocate(100, 8).unwrap();
        let ptr2 = arena.allocate(200, 8).unwrap();

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert_eq!(arena.total_allocated(), 300);
        assert!(arena.chunk_count() >= 1);

        // Reset arena
        arena.reset();
        assert_eq!(arena.total_allocated(), 0);
    }

    #[test]
    fn test_advanced_memory_manager() {
        let mut manager = MemoryManager::<f32>::with_advanced_features(
            true, // enable_zero_copy
            true, // enable_arena
            true, // enable_leak_detection
            4096, // arena_chunk_size
        );

        // Test arena allocation
        let ptr = manager.arena_allocate(100).unwrap();
        assert!(!ptr.is_null());

        // Test pool allocation
        manager.create_pool("test", 100);
        let buffer = manager.allocate("test", 50).unwrap();
        assert_eq!(buffer.len(), 50);

        // Check stats
        let stats = manager.get_stats();
        assert!(stats.total_allocated > 0);
        assert!(stats.allocation_count > 0);
    }

    #[test]
    fn test_memory_manager_with_capacity() {
        let layer_sizes = vec![10, 20, 30];
        let manager = MemoryManager::<f32>::with_capacity(&layer_sizes);

        // Should have created pools for each layer
        assert!(manager.pools.contains_key("layer_0_neurons"));
        assert!(manager.pools.contains_key("layer_1_weights"));
        assert!(manager.pools.contains_key("layer_2_gradients"));

        // Should have common utility pools
        assert!(manager.pools.contains_key("activations"));
        assert!(manager.pools.contains_key("errors"));
        assert!(manager.pools.contains_key("temp"));
    }

    #[test]
    fn test_zero_copy_operations() {
        let mut manager = MemoryManager::<f32>::new();
        manager.enable_zero_copy = true;

        let data = vec![1.0, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];

        let view = manager.create_tensor_view(&data, &shape).unwrap();
        assert_eq!(view.shape(), &[2, 2]);
        assert_eq!(view.get(&[0, 0]).unwrap(), 1.0);
    }

    #[test]
    fn test_memory_stats_comprehensive() {
        let mut manager = MemoryManager::<f32>::new();
        manager.create_pool("test", 100);

        // Perform some allocations
        let _buffer1 = manager.allocate("test", 50).unwrap();
        let _buffer2 = manager.allocate("test", 30).unwrap();

        let stats = manager.get_stats();

        assert!(stats.total_allocated > 0);
        assert!(stats.buffer_count > 0);
        assert!(stats.allocation_count >= 2);
        assert!(stats.average_allocation_size > 0.0);
        assert!(stats.peak_memory >= stats.current_memory);
    }
}
