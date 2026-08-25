import { useState, useRef, useEffect } from "react";
import { Send, Loader2, CheckCircle2, XCircle, Wrench, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// ── Wire types (mirror of uh-core/src/transport.rs) ─────────────

interface WsRequest {
  kind: "request";
  id: string;
  method: string;
  params: unknown;
}
interface WsResponse {
  kind: "response";
  id: string;
  result: unknown;
  error: { code: number; message: string; data: unknown } | null;
}
interface WsEvent {
  kind: "event";
  event: string;
  data: unknown;
}
type WsIncoming = WsResponse | WsEvent;

interface TurnStartEvent {
  turn_id: string;
  user_message: string;
}
interface AssistantMessageEvent {
  message: {
    role: "assistant";
    content: string;
    tool_calls?: { id: string; name: string; arguments: unknown }[];
  };
}
interface ToolCallEvent {
  tool_call: { id: string; name: string; arguments: unknown };
}
interface ToolResultEvent {
  tool_call_id: string;
  result: { tool_call_id: string; content: string; is_error: boolean; display?: unknown };
}
interface TurnEndEvent {
  turn_id: string;
  reason: string;
}

type DisplayItem =
  | { kind: "user"; content: string }
  | { kind: "assistant"; content: string }
  | { kind: "tool_call"; name: string; args: unknown }
  | { kind: "tool_result"; content: string; is_error: boolean }
  | { kind: "turn_end"; reason: string };

interface Props {
  wsRef: React.MutableRefObject<WebSocket | null>;
  onConnected: (connected: boolean) => void;
}

export function Chat({ wsRef, onConnected }: Props) {
  const [items, setItems] = useState<DisplayItem[]>([]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [connected, setConnected] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [items]);

  const send = (text: string) => {
    if (!text.trim() || !wsRef.current) return;
    setItems((prev) => [...prev, { kind: "user", content: text }]);
    setInput("");
    setBusy(true);
    const req: WsRequest = {
      kind: "request",
      id: crypto.randomUUID(),
      method: "send_message",
      params: { content: text },
    };
    wsRef.current.send(JSON.stringify(req));
  };

  // Expose a way for the App to dispatch WS events into the chat
  // by listening to incoming messages.
  useEffect(() => {
    const ws = wsRef.current;
    if (!ws) return;
    const handler = (ev: MessageEvent) => {
      let msg: WsIncoming;
      try {
        msg = JSON.parse(ev.data);
      } catch {
        return;
      }
      if (msg.kind === "event") {
        onWsEvent(msg);
      } else if (msg.kind === "response") {
        setBusy(false);
      }
    };
    const onOpen = () => {
      setConnected(true);
      onConnected(true);
    };
    const onClose = () => {
      setConnected(false);
      onConnected(false);
    };
    ws.addEventListener("message", handler);
    ws.addEventListener("open", onOpen);
    ws.addEventListener("close", onClose);
    if (ws.readyState === WebSocket.OPEN) {
      setConnected(true);
      onConnected(true);
    } else if (ws.readyState === WebSocket.CLOSED) {
      setConnected(false);
      onConnected(false);
    }
    return () => {
      ws.removeEventListener("message", handler);
      ws.removeEventListener("open", onOpen);
      ws.removeEventListener("close", onClose);
    };
  }, [wsRef, onConnected]);

  function onWsEvent(msg: WsEvent) {
    switch (msg.event) {
      case "turn_start": {
        const e = msg.data as TurnStartEvent;
        setItems((p) => [...p, { kind: "user", content: e.user_message }]);
        break;
      }
      case "assistant_message": {
        const e = msg.data as AssistantMessageEvent;
        if (e.message.content) {
          setItems((p) => [...p, { kind: "assistant", content: e.message.content }]);
        }
        break;
      }
      case "tool_call": {
        const e = msg.data as ToolCallEvent;
        setItems((p) => [...p, { kind: "tool_call", name: e.tool_call.name, args: e.tool_call.arguments }]);
        break;
      }
      case "tool_result": {
        const e = msg.data as ToolResultEvent;
        setItems((p) => [...p, { kind: "tool_result", content: e.result.content, is_error: e.result.is_error }]);
        break;
      }
      case "turn_end": {
        const e = msg.data as TurnEndEvent;
        setItems((p) => [...p, { kind: "turn_end", reason: e.reason }]);
        setBusy(false);
        break;
      }
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-4 py-6">
        <div className="mx-auto max-w-3xl space-y-3">
          {items.length === 0 && (
            <div className="rounded-lg border border-dashed border-border/60 bg-card/30 p-8 text-center">
              <p className="text-sm text-muted-foreground">
                {connected
                  ? "Ready. Send a message to start."
                  : "Connecting to daemon…"}
              </p>
            </div>
          )}
          {items.map((item, i) => (
            <ChatItem key={i} item={item} />
          ))}
          {busy && (
            <div className="flex items-center gap-2 px-3 text-xs text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" />
              <span>thinking…</span>
            </div>
          )}
        </div>
      </div>
      <div className="border-t border-border bg-card/40 p-4">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            send(input);
          }}
          className="mx-auto flex max-w-3xl gap-2"
        >
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder={
              connected
                ? "Tell the AI what to do (e.g. \"rename fn bar to baz in src/foo.rs\")"
                : "Waiting for daemon…"
            }
            disabled={!connected || busy}
            className="flex-1"
          />
          <Button type="submit" disabled={!connected || busy || !input.trim()}>
            <Send className="h-4 w-4" /> Send
          </Button>
        </form>
      </div>
    </div>
  );
}

function ChatItem({ item }: { item: DisplayItem }) {
  if (item.kind === "user") {
    return (
      <div className="flex justify-end">
        <div className="max-w-2xl rounded-lg bg-primary/15 px-4 py-2.5 text-sm ring-1 ring-primary/20">
          {item.content}
        </div>
      </div>
    );
  }
  if (item.kind === "assistant") {
    return (
      <div className="flex justify-start">
        <div className="max-w-2xl rounded-lg border border-border/60 bg-card px-4 py-2.5 text-sm leading-relaxed">
          {item.content}
        </div>
      </div>
    );
  }
  if (item.kind === "tool_call") {
    return (
      <Card className="bg-amber-500/5 border-amber-500/30">
        <CardContent className="flex items-start gap-2 p-3 text-xs">
          <Wrench className="mt-0.5 h-3.5 w-3.5 text-amber-400" />
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <Badge variant="warning" className="font-mono text-[10px]">
                {item.name}
              </Badge>
            </div>
            <pre className="mt-1 overflow-x-auto rounded bg-background/40 p-2 font-mono text-[11px] text-foreground/80">
              {JSON.stringify(item.args, null, 2)}
            </pre>
          </div>
        </CardContent>
      </Card>
    );
  }
  if (item.kind === "tool_result") {
    return (
      <Card
        className={cn(
          "ml-6",
          item.is_error ? "bg-red-500/5 border-red-500/30" : "bg-emerald-500/5 border-emerald-500/30",
        )}
      >
        <CardContent className="flex items-start gap-2 p-3 text-xs">
          {item.is_error ? (
            <XCircle className="mt-0.5 h-3.5 w-3.5 text-red-400" />
          ) : (
            <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 text-emerald-400" />
          )}
          <pre className="flex-1 overflow-x-auto whitespace-pre-wrap font-mono text-[11px] text-foreground/85">
            {item.content}
          </pre>
        </CardContent>
      </Card>
    );
  }
  if (item.kind === "turn_end") {
    return (
      <div className="flex items-center gap-1.5 px-2 py-1 font-mono text-[10px] text-muted-foreground">
        <ChevronRight className="h-3 w-3" />
        turn ended: <span className="text-foreground/80">{item.reason}</span>
      </div>
    );
  }
  return null;
}
