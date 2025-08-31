/**
 * Core AI integration module
 * Handles MCP server setup and basic integration
 */

import { execSync } from 'child_process';
import { promises as fs } from 'fs';
import path from 'path';

class AIIntegrationCore {
  constructor(options = {}) {
    this.autoSetup = options.autoSetup || false;
    this.forceSetup = options.forceSetup || false;
    this.workingDir = options.workingDir || process.cwd();
  }

  /**
   * Check if AI CLI is available
   */
  async isAIAvailable() {
    try {
      execSync('claude --version', { stdio: 'ignore' });
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Add ruv-swarm MCP server to AI platform
   */
  async addMcpServer() {
    if (!(await this.isAIAvailable())) {
      throw new Error(
        'AI platform CLI not found. Please install the appropriate AI platform CLI'
      );
    }

    try {
      // Add ruv-swarm MCP server using stdio (no port needed)
      const mcpCommand = 'ai mcp add ruv-swarm npx ruv-swarm mcp start';
      execSync(mcpCommand, { stdio: 'inherit', cwd: this.workingDir });
      return { success: true, message: 'Added ruv-swarm MCP server to AI platform (stdio)' };
    } catch (error) {
      throw new Error(`Failed to add MCP server: ${error.message}`);
    }
  }

  /**
   * Check if integration files already exist
   */
  async checkExistingFiles() {
    try {
      await fs.access(path.join(this.workingDir, 'claude.md'));
      await fs.access(path.join(this.workingDir, '.claude/commands'));
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Initialize AI integration
   */
  async initialize() {
    console.log('🔧 Initializing AI integration...');

    // Check if files exist (unless force setup)
    if (!this.forceSetup && (await this.checkExistingFiles())) {
      console.log('   ℹ️  AI integration files already exist (use --force to regenerate)');
      return { success: true, message: 'Integration files already exist' };
    }

    try {
      const results = {
        core: await this.addMcpServer(),
        success: true
      };

      console.log('✅ AI integration initialized successfully');
      return results;
    } catch (error) {
      console.error('❌ Failed to initialize AI integration:', error.message);
      throw error;
    }
  }

  /**
   * Invoke AI with a prompt (supports both secure and legacy modes)
   */
  async invokeAIWithPrompt(prompt, options = {}) {
    if (!prompt || !prompt.trim()) {
      throw new Error('No prompt provided');
    }

    if (!(await this.isAIAvailable())) {
      throw new Error('AI platform CLI not found');
    }

    // Default behavior for backward compatibility (legacy mode)
    const addPermissions = options.secure !== true;
    const permissions = addPermissions ? ' --dangerously-skip-permissions' : '';
    const aiCommand = `ai "${prompt.trim()}"${permissions}`;

    try {
      execSync(aiCommand, { stdio: 'inherit', cwd: this.workingDir });
      return { success: true, message: 'AI invocation completed' };
    } catch (error) {
      throw new Error(`AI invocation failed: ${error.message}`);
    }
  }
}

export { AIIntegrationCore };
