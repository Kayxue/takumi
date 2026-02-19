import {
  Code2Icon,
  DownloadIcon,
  EyeIcon,
  Loader2Icon,
  RotateCcwIcon,
  Wand2Icon,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";
import type { z } from "zod/mini";
import {
  messageSchema,
  type RenderMessageInput,
  type renderResultSchema,
} from "~/playground/schema";
import { compressCode, decompressCode } from "~/playground/share";
import { templates } from "~/playground/templates";
import TakumiWorker from "~/playground/worker?worker";
import { Button } from "../ui/button";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "../ui/resizable";
import { ComponentEditor } from "./component-editor";

const mobileViewportWidth = 768;

function useIsMobile() {
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const resize = () => {
      setIsMobile(window.innerWidth < mobileViewportWidth);
    };
    resize();
    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, []);

  return isMobile;
}

export default function Playground() {
  const [code, setCode] = useState<string>();
  const [rendered, setRendered] =
    useState<z.infer<typeof renderResultSchema>["result"]>();
  const [isReady, setIsReady] = useState(false);
  const [isFormatting, setIsFormatting] = useState(false);
  const currentRequestIdRef = useRef(0);

  const workerRef = useRef<Worker | undefined>(undefined);
  const isMobile = useIsMobile();
  const [searchParams, setSearchParams] = useSearchParams();
  const [activeTab, setActiveTab] = useState<"code" | "preview">("code");

  const codeQuery = searchParams.get("code");

  useEffect(() => {
    if (code) return;

    if (codeQuery) decompressCode(codeQuery).then(setCode);
    else setCode(templates[0].code);
  }, [codeQuery, code]);

  useEffect(() => {
    if (!code) return;

    if (code === templates[0].code) {
      return setSearchParams(
        (prev) => {
          prev.delete("code");

          return prev;
        },
        { replace: true },
      );
    }

    compressCode(code).then((base64) => {
      setSearchParams(
        (prev) => {
          prev.set("code", base64);

          return prev;
        },
        { replace: true },
      );
    });
  }, [code, setSearchParams]);

  useEffect(() => {
    const worker = new TakumiWorker();

    worker.onmessage = (event: MessageEvent) => {
      const message = messageSchema.parse(event.data);

      switch (message.type) {
        case "ready": {
          setIsReady(true);
          break;
        }
        case "render-request": {
          throw new Error("request is not possible for response");
        }
        case "render-result": {
          if (message.result.id === currentRequestIdRef.current) {
            setRendered(message.result);
          }
          break;
        }
        default: {
          message satisfies never;
        }
      }
    };

    workerRef.current = worker;

    return () => {
      worker.terminate();
      workerRef.current = undefined;
      setIsReady(false);
    };
  }, []);

  useEffect(() => {
    if (isReady && code) {
      const requestId = currentRequestIdRef.current + 1;
      currentRequestIdRef.current = requestId;
      workerRef.current?.postMessage({
        type: "render-request",
        id: requestId,
        code,
      } satisfies RenderMessageInput);
    }
  }, [isReady, code]);

  const loadTemplate = (templateCode: string) => {
    setCode(templateCode);
    setActiveTab("code");
  };

  const formatCode = async () => {
    if (!code) return;
    try {
      setIsFormatting(true);
      const [prettier, prettierPluginEstree, prettierPluginTypeScript] =
        await Promise.all([
          import("prettier/standalone"),
          import("prettier/plugins/estree"),
          import("prettier/plugins/typescript"),
        ]);

      const formatted = await prettier.format(code, {
        parser: "typescript",
        plugins: [prettierPluginEstree, prettierPluginTypeScript],
      });

      setCode(formatted);
    } catch (error) {
      console.error("Failed to format code:", error);
    } finally {
      setIsFormatting(false);
    }
  };

  return (
    <div className="flex h-[calc(100dvh-3.5rem)] flex-col bg-[#09090b]">
      {isMobile ? (
        <MobileView
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          loadTemplate={loadTemplate}
          formatCode={formatCode}
          isFormatting={isFormatting}
          code={code}
          setCode={setCode}
          rendered={rendered}
          isMobile={isMobile}
        />
      ) : (
        <ResizablePanelGroup orientation="horizontal">
          <ResizablePanel defaultSize={60} minSize={30}>
            <CodePanel
              code={code}
              setCode={setCode}
              formatCode={formatCode}
              isFormatting={isFormatting}
              loadTemplate={loadTemplate}
            />
          </ResizablePanel>
          <ResizableHandle
            withHandle
            className="bg-zinc-800/50 hover:bg-zinc-700 transition-colors"
          />
          <ResizablePanel defaultSize={40} minSize={30}>
            <PreviewPanel isMobile={isMobile} rendered={rendered} />
          </ResizablePanel>
        </ResizablePanelGroup>
      )}
    </div>
  );
}

function MobileView({
  activeTab,
  setActiveTab,
  loadTemplate,
  formatCode,
  isFormatting,
  code,
  setCode,
  rendered,
  isMobile,
}: {
  activeTab: "code" | "preview";
  setActiveTab: React.Dispatch<React.SetStateAction<"code" | "preview">>;
  loadTemplate: (templateCode: string) => void;
  formatCode: () => void;
  isFormatting: boolean;
  code: string | undefined;
  setCode: React.Dispatch<React.SetStateAction<string | undefined>>;
  rendered: z.infer<typeof renderResultSchema>["result"] | undefined;
  isMobile: boolean;
}) {
  return (
    <>
      <div className="flex shrink-0 gap-2 overflow-x-auto border-b border-zinc-800/80 bg-background p-2">
        <Button
          variant={activeTab === "code" ? "default" : "secondary"}
          className="flex-1 h-9 rounded-full text-xs font-semibold"
          onClick={() => setActiveTab("code")}
        >
          <Code2Icon className="mr-2 h-4 w-4" />
          Code
        </Button>
        <Button
          variant={activeTab === "preview" ? "default" : "secondary"}
          className="flex-1 h-9 rounded-full text-xs font-semibold"
          onClick={() => setActiveTab("preview")}
        >
          <EyeIcon className="mr-2 h-4 w-4" />
          Preview
        </Button>
      </div>
      {activeTab === "code" && (
        <div className="scrollbar-hide flex shrink-0 items-center overflow-x-auto border-b border-zinc-800/80 bg-zinc-950/40 px-3 py-2 shadow-sm gap-2">
          <span className="mr-1 whitespace-nowrap text-[10px] font-bold uppercase tracking-widest text-zinc-500">
            Templates
          </span>
          {templates?.map((t) => (
            <Button
              key={t.name}
              variant="outline"
              size="sm"
              className="h-7 whitespace-nowrap rounded-full border-zinc-700/80 bg-[#121214] text-xs font-medium hover:bg-zinc-800 hover:text-white"
              onClick={() => loadTemplate(t.code)}
            >
              {t.name}
            </Button>
          ))}
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto h-7 whitespace-nowrap text-xs text-zinc-400 hover:text-zinc-200 font-medium"
            onClick={formatCode}
            disabled={isFormatting}
          >
            {isFormatting ? (
              <Loader2Icon className="mr-1.5 h-3 w-3 animate-spin" />
            ) : (
              <Wand2Icon className="mr-1.5 h-3 w-3" />
            )}
            {isFormatting ? "Formatting..." : "Format"}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 whitespace-nowrap text-xs text-red-500 hover:bg-red-500/10 hover:text-red-400 font-medium"
            onClick={() => loadTemplate(templates[0].code)}
          >
            <RotateCcwIcon className="mr-1.5 h-3 w-3" />
            Reset
          </Button>
        </div>
      )}
      <div className="relative min-h-0 flex-1">
        {activeTab === "code" ? (
          <CodePanel
            code={code}
            setCode={setCode}
            formatCode={formatCode}
            isFormatting={isFormatting}
            loadTemplate={loadTemplate}
          />
        ) : (
          <PreviewPanel isMobile={isMobile} rendered={rendered} />
        )}
      </div>
    </>
  );
}

function CodePanel({
  code,
  setCode,
  formatCode,
  isFormatting,
  loadTemplate,
}: {
  code: string | undefined;
  setCode: React.Dispatch<React.SetStateAction<string | undefined>>;
  formatCode: () => void;
  isFormatting: boolean;
  loadTemplate: (templateCode: string) => void;
}) {
  return (
    <div className="flex h-full flex-col bg-zinc-950/50">
      <div className="flex h-11 shrink-0 items-center justify-between border-b border-zinc-800 bg-zinc-950/40 px-4">
        <p className="text-sm font-medium text-zinc-300">Editor</p>
        <div className="flex gap-2 items-center">
          <div className="hidden sm:flex gap-2 items-center mr-2 border-r border-zinc-800 pr-4">
            <span className="text-[10px] uppercase font-bold text-zinc-600 tracking-wider">
              Examples:
            </span>
            {templates?.map((t) => (
              <Button
                key={t.name}
                variant="outline"
                size="sm"
                className="h-7 text-xs border-zinc-800 hover:bg-zinc-800"
                onClick={() => loadTemplate(t.code)}
              >
                {t.name}
              </Button>
            ))}
          </div>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-xs text-zinc-400 hover:text-zinc-200"
            onClick={formatCode}
            disabled={isFormatting}
          >
            {isFormatting ? (
              <Loader2Icon className="mr-1 h-3 w-3 animate-spin" />
            ) : (
              <Wand2Icon className="mr-1 h-3 w-3" />
            )}
            {isFormatting ? "Formatting..." : "Format"}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-xs text-zinc-400 hover:text-zinc-200"
            onClick={() => setCode(templates[0].code)}
          >
            <RotateCcwIcon className="mr-1 h-3 w-3" />
            Reset
          </Button>
        </div>
      </div>
      <div className="relative min-h-0 flex-1">
        {code && <ComponentEditor code={code} setCode={setCode} />}
      </div>
    </div>
  );
}

function PreviewPanel({
  isMobile,
  rendered,
}: {
  isMobile: boolean;
  rendered: z.infer<typeof renderResultSchema>["result"] | undefined;
}) {
  return (
    <div className="flex h-full flex-col bg-[#09090b]">
      {!isMobile && (
        <div className="flex h-11 shrink-0 items-center justify-between border-b border-zinc-800 bg-zinc-950/40 px-4">
          <p className="text-sm font-medium text-zinc-300">Preview</p>
        </div>
      )}
      <div className="flex-1 w-full overflow-y-auto">
        {rendered && <RenderPreview result={rendered} />}
      </div>
    </div>
  );
}

function RenderPreview({
  result,
}: {
  result: z.infer<typeof renderResultSchema>["result"];
}) {
  if (result.status === "error") {
    return (
      <div className="flex h-full w-full flex-col items-center justify-center p-8 bg-red-950/10">
        <div className="bg-red-950/30 border border-red-900/50 p-6 rounded-xl flex flex-col items-center max-w-2xl w-full">
          <p className="mb-4 text-xl font-bold text-red-400">Error Rendering</p>
          <pre className="w-full whitespace-pre-wrap rounded-lg bg-black/60 p-5 text-xs text-red-300 shadow-inner overflow-x-auto leading-relaxed border border-red-900/20">
            {result.message}
          </pre>
        </div>
      </div>
    );
  }

  return (
    <div className="relative flex h-full w-full flex-col items-center justify-center gap-8 p-4 sm:p-8">
      <div
        className="relative shadow-2xl overflow-hidden rounded-xl border border-zinc-800/60 transition-all hover:shadow-emerald-500/5 hover:border-zinc-700/80"
        style={{
          backgroundImage:
            "radial-gradient(circle at 10px 10px, #18181b 2%, transparent 0%), radial-gradient(circle at 25px 25px, #18181b 2%, transparent 0%)",
          backgroundSize: "30px 30px",
          backgroundColor: "#09090b",
        }}
      >
        <img
          src={result.dataUrl}
          alt="Rendered component"
          className="object-contain"
          style={{
            maxWidth: "100%",
            maxHeight: "calc(100vh - 14rem)",
          }}
        />
      </div>
      <div className="flex items-center gap-3 text-xs font-medium">
        <div className="flex h-9 items-center rounded-full border border-zinc-800/80 bg-zinc-900/80 px-4 text-zinc-400 shadow-lg backdrop-blur-md">
          Format
          <span className="ml-2 rounded bg-zinc-950 px-1.5 py-0.5 text-zinc-100 font-mono">
            {result.options.format.toUpperCase()}
          </span>
        </div>
        <div className="flex h-9 items-center rounded-full border border-emerald-900/30 bg-emerald-950/10 px-4 text-zinc-400 shadow-lg backdrop-blur-md">
          <div className="mr-2 h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)]" />
          Time
          <span className="ml-2 rounded bg-zinc-950 px-1.5 py-0.5 text-emerald-400 font-mono">
            {Math.round(result.duration)}ms
          </span>
        </div>
        <Button
          variant="default"
          size="sm"
          className="h-9 rounded-full bg-zinc-100 px-4! font-semibold text-zinc-900 shadow-lg transition-transform hover:scale-105 hover:bg-white active:scale-95"
          onClick={() => {
            const link = document.createElement("a");
            link.href = result.dataUrl;
            link.download = `takumi-image.${result.options.format}`;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
          }}
        >
          <DownloadIcon className="h-3.5 w-3.5" />
          Download
        </Button>
      </div>
    </div>
  );
}
