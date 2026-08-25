import { useEffect, useRef, useState } from "react";
import { Boxes, Code2, Cpu, Wifi, WifiOff } from "lucide-react";
import { Chat } from "./components/Chat";
import { Button } from "./components/ui/button";
import { Badge } from "./components/ui/badge";
import { TooltipProvider } from "./components/ui/tooltip";
import { Review } from "./review";

function isReviewHash() {
  return typeof window !== "undefined" && window.location.hash === "#/review";
}

export function App() {
  const [isReview, setIsReview] = useState(isReviewHash);
  const [connected, setConnected] = useState(false);
  const [llmInfo, setLlmInfo] = useState<{ provider: string; model: string } | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const handler = () => setIsReview(isReviewHash());
    window.addEventListener("hashchange", handler);
    return () => window.removeEventListener("hashchange", handler);
  }, []);

  if (isReview) {
    return (
      <div className="flex h-screen flex-col bg-background text-foreground">
        <Review />
      </div>
    );
  }

  // Connect to daemon WS.
  useEffect(() => {
    const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${proto}//${window.location.host}/ws`;
    const ws = new WebSocket(url);
    wsRef.current = ws;
    return () => {
      ws.close();
    };
  }, []);

  // Fetch /api/llm for the header.
  useEffect(() => {
    fetch("/api/llm")
      .then((r) => r.json())
      .then((d) => setLlmInfo(d))
      .catch(() => setLlmInfo(null));
  }, []);

  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex h-screen flex-col bg-background text-foreground">
        <Header
          connected={connected}
          llmInfo={llmInfo}
          onOpenReview={() => (window.location.hash = "#/review")}
        />
        <main className="flex-1 overflow-hidden">
          <Chat wsRef={wsRef} onConnected={setConnected} />
        </main>
        <Footer />
      </div>
    </TooltipProvider>
  );
}

function Header({
  connected,
  llmInfo,
  onOpenReview,
}: {
  connected: boolean;
  llmInfo: { provider: string; model: string } | null;
  onOpenReview: () => void;
}) {
  return (
    <header className="flex items-center justify-between border-b border-border bg-card/40 px-6 py-3">
      <div className="flex items-center gap-3">
        <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary/10 text-primary ring-1 ring-primary/20">
          <Boxes className="h-4 w-4" />
        </div>
        <div>
          <h1 className="text-sm font-semibold leading-none tracking-tight">
            Ultimate <span className="text-primary">Harness</span>
          </h1>
          <p className="mt-0.5 font-mono text-[10px] text-muted-foreground">
            {llmInfo
              ? `${llmInfo.provider} / ${llmInfo.model}`
              : "loading llm info…"}
          </p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Badge variant="outline" className="font-mono text-[10px]">
          v0.0.3
        </Badge>
        <ConnectionDot connected={connected} />
        <Button
          size="sm"
          variant="outline"
          onClick={onOpenReview}
          className="h-7 gap-1.5 px-2 text-[11px]"
          title="Open Rust code viewer (dev tool) at /#/review"
        >
          <Code2 className="h-3 w-3" /> /review
        </Button>
      </div>
    </header>
  );
}

function ConnectionDot({ connected }: { connected: boolean }) {
  return (
    <span
      className={`inline-flex items-center gap-1 font-mono text-[10px] ${
        connected ? "text-emerald-400" : "text-amber-400"
      }`}
      title={connected ? "daemon connected" : "daemon disconnected"}
    >
      {connected ? <Wifi className="h-3 w-3" /> : <WifiOff className="h-3 w-3" />}
    </span>
  );
}

function Footer() {
  return (
    <footer className="flex items-center gap-2 border-t border-border bg-card/40 px-6 py-1.5 font-mono text-[10px] text-muted-foreground">
      <Cpu className="h-3 w-3" />
      <span>
        v0.0.3 — daemon + WebSocket. See <code>uh.toml.example</code> for config.
      </span>
    </footer>
  );
}
