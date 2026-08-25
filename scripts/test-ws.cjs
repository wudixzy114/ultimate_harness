// Quick WS test for uh-daemon.
// Usage: node scripts/test-ws.cjs [port]
const port = process.argv[2] || 3090;
const WebSocket = require("ws");
const ws = new WebSocket(`ws://127.0.0.1:${port}/ws`);

const events = [];
let received = 0;

ws.on("open", () => {
  console.log("connected");
  // List tools first
  ws.send(JSON.stringify({
    kind: "request",
    id: crypto.randomUUID(),
    method: "list_tools",
    params: {},
  }));
  setTimeout(() => {
    console.log("\n→ send_message: 'just say hi'");
    ws.send(JSON.stringify({
      kind: "request",
      id: crypto.randomUUID(),
      method: "send_message",
      params: { content: "just say hi" },
    }));
  }, 500);
  // 8s timeout
  setTimeout(() => {
    console.log("\n--- final events ---");
    for (const e of events) console.log(" ", e);
    process.exit(0);
  }, 8000);
});

ws.on("message", (data) => {
  received++;
  const msg = JSON.parse(data);
  if (msg.kind === "event") {
    events.push(`event: ${msg.event} ${JSON.stringify(msg.data).slice(0, 100)}`);
    console.log(`  [event] ${msg.event}`, JSON.stringify(msg.data).slice(0, 100));
  } else if (msg.kind === "response") {
    events.push(`response: ${JSON.stringify(msg.result || msg.error).slice(0, 100)}`);
    console.log(`  [response]`, JSON.stringify(msg.result || msg.error).slice(0, 100));
  }
});

ws.on("error", (e) => console.log("error:", e.message));
ws.on("close", () => console.log("closed (total received:", received, ")"));
