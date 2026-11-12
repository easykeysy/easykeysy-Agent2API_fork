//! MCP （ to client when by to ，
//! `node vision-bridge.js` stdio MCP supports client）。
//!
//! use ： use need need filepath（ or URL / data URI）， in 
//! file base64 data URI， to MCP —— 
//! 「 in up down base64 」 。
//!
//! config（ by client MCP config when ）：
//! AGENT2API_VISION_ENDPOINT MCP URL（ /mcp/vision）
//! AGENT2API_VISION_KEY API Key（Bearer）
//! AGENT2API_VISION_TIMEOUT_MS request when （ms，optional；default 300000）。
//! 「 when + 60s 」，client when 。

pub const VISION_BRIDGE_SCRIPT: &str = r#"'use strict';
/*
 * Agent2API MCP （stdio，Node >= 18）。
 * ：AGENT2API_VISION_ENDPOINT / AGENT2API_VISION_KEY（ by Agent2API ）。
 */

var fs = require('node:fs');
var path = require('node:path');
var readline = require('node:readline');

var ENDPOINT = (process.env.AGENT2API_VISION_ENDPOINT || '').replace(/\/$/, '');
var API_KEY = process.env.AGENT2API_VISION_KEY || '';
var CALL_TIMEOUT_MS = Number(process.env.AGENT2API_VISION_TIMEOUT_MS) > 0
  ? Number(process.env.AGENT2API_VISION_TIMEOUT_MS)
  : 300000;

var MIME_BY_EXT = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.webp': 'image/webp',
  '.bmp': 'image/bmp',
  '.svg': 'image/svg+xml',
};

var TOOLS = [
  {
    name: 'image_analyze',
    title: 'Image analysis',
    description:
      'Analyze 1-10 images with the vision model configured by the Agent2API administrator. ' +
      'Each image may be a LOCAL FILE PATH on this machine, an http(s) URL, or a data URI; ' +
      'local files are read and converted automatically. The prompt describes what to look for.',
    inputSchema: {
      type: 'object',
      properties: {
        images: {
          type: 'array',
          description: 'Images to analyze (1-10): local file paths, http(s) URLs or data URIs.',
          items: { type: 'string', minLength: 1 },
          minItems: 1,
          maxItems: 10,
        },
        prompt: {
          type: 'string',
          description: 'What to look for in the images.',
          minLength: 1,
        },
      },
      required: ['images', 'prompt'],
      additionalProperties: false,
    },
  },
];

function log(message) {
  process.stderr.write('[vision-bridge] ' + message + '\n');
}

function rpcResult(id, result) {
  return { jsonrpc: '2.0', id: id, result: result };
}

function rpcError(id, code, message) {
  return { jsonrpc: '2.0', id: id, error: { code: code, message: message } };
}

function toRemoteImage(ref) {
  var value = String(ref || '').trim();
  if (/^data:image\//i.test(value)) return value;
  if (/^https?:\/\//i.test(value)) return value;
  var stat = fs.statSync(value);
  if (!stat.isFile()) throw new Error('not a file: ' + value);
  var ext = (path.extname(value) || '').toLowerCase();
  var mime = MIME_BY_EXT[ext] || 'image/png';
  var buffer = fs.readFileSync(value);
  if (buffer.length === 0) throw new Error('empty file: ' + value);
  return 'data:' + mime + ';base64,' + buffer.toString('base64');
}

function toolResult(text) {
  return {
    resultType: 'complete',
    content: [{ type: 'text', text: text }],
  };
}

function toolError(message) {
  return {
    resultType: 'complete',
    content: [{ type: 'text', text: message }],
    isError: true,
  };
}

async function callTool(name, args, requestId) {
  if (name !== 'image_analyze') {
    return rpcError(requestId, -32602, 'Unknown tool: ' + String(name));
  }
  var images = Array.isArray(args && args.images) ? args.images : [];
  var prompt = typeof (args && args.prompt) === 'string' ? args.prompt.trim() : '';
  if (images.length < 1 || images.length > 10) {
    return rpcError(requestId, -32602, 'images must contain between 1 and 10 image references');
  }
  if (!prompt) return rpcError(requestId, -32602, 'prompt must be a non-empty string');
  if (!ENDPOINT || !API_KEY) {
    return rpcResult(requestId, toolError('Bridge is not configured (missing AGENT2API_VISION_ENDPOINT / AGENT2API_VISION_KEY); reinstall from the Agent2API panel.'));
  }

  var converted;
  try {
    converted = images.map(toRemoteImage);
  } catch (error) {
    return rpcResult(requestId, toolError('Cannot read image: ' + ((error && error.message) || String(error))));
  }

  var response;
  try {
    response = await fetch(ENDPOINT, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        authorization: 'Bearer ' + API_KEY,
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'tools/call',
        params: {
          name: 'image_analyze',
          arguments: { images: converted, prompt: prompt },
        },
      }),
      signal: AbortSignal.timeout(CALL_TIMEOUT_MS),
    });
  } catch (error) {
    return rpcResult(requestId, toolError('Cannot reach the Agent2API vision endpoint: ' + ((error && error.message) || String(error))));
  }

  var payload = null;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }
  if (!response.ok || !payload || !payload.result) {
    var detail = payload && payload.error ? payload.error.message : 'HTTP ' + response.status;
    return rpcResult(requestId, toolError('Vision endpoint error: ' + detail));
  }
  return rpcResult(requestId, payload.result);
}

async function dispatch(message) {
  if (!message || typeof message !== 'object' || message.jsonrpc !== '2.0' || typeof message.method !== 'string') {
    return rpcError(null, -32600, 'Invalid JSON-RPC 2.0 request');
  }
  var method = message.method;
  if (method.indexOf('notifications/') === 0) return null;
  if (message.id === undefined || message.id === null) return null;
  var id = message.id;
  var params = message.params || {};

  if (method === 'initialize') {
    return rpcResult(id, {
      protocolVersion: typeof params.protocolVersion === 'string' ? params.protocolVersion : '2025-03-26',
      capabilities: { tools: {} },
      serverInfo: { name: 'vision-bridge', title: 'Vision Bridge', version: '1.0.0' },
      instructions:
        'Vision analysis bridge. Pass local file paths on this machine (or http(s) URLs / data URIs); files are read and converted automatically.',
    });
  }
  if (method === 'ping') return rpcResult(id, {});
  if (method === 'tools/list') {
    return rpcResult(id, { tools: TOOLS });
  }
  if (method === 'tools/call') {
    return await callTool(String(params.name || ''), params.arguments || {}, id);
  }
  return rpcError(id, -32601, 'Method not found: ' + method);
}

function main() {
  if (!ENDPOINT || !API_KEY) {
    log('AGENT2API_VISION_ENDPOINT / AGENT2API_VISION_KEY not set; tools/call will return an error until reinstalled.');
  } else {
    log('bridge ready -> ' + ENDPOINT);
  }
  var rl = readline.createInterface({ input: process.stdin, terminal: false });
  rl.on('line', function (line) {
    var text = line.trim();
    if (!text) return;
    var message;
    try {
      message = JSON.parse(text);
    } catch (error) {
      process.stdout.write(JSON.stringify(rpcError(null, -32700, 'Parse error')) + '\n');
      return;
    }
    void dispatch(message).then(function (response) {
      if (response) process.stdout.write(JSON.stringify(response) + '\n');
    }).catch(function (error) {
      log('dispatch failed: ' + ((error && error.message) || String(error)));
      var id = message && message.id !== undefined ? message.id : null;
      process.stdout.write(JSON.stringify(rpcError(id, -32603, 'Internal error')) + '\n');
    });
  });
  rl.on('close', function () {
    process.exit(0);
  });
}

main();
"#;
