type CssLibraryCardProps = {
  libraryName: string;
  title: string;
  description: string;
};

export function CssLibraryCard({
  libraryName,
  title,
  description,
}: CssLibraryCardProps) {
  return (
    <div className="flex h-full w-full items-center justify-center bg-slate-950 p-16">
      <div className="flex w-[960px] items-center gap-10 overflow-hidden rounded-[32px] border border-white/10 bg-white shadow-2xl">
        <div className="flex h-[420px] w-[360px] flex-col justify-between rounded-r-3xl bg-linear-to-br from-cyan-400 via-blue-500 to-indigo-700 p-8 text-white">
          <div className="flex flex-col space-y-4">
            <p className="text-sm font-semibold uppercase tracking-[0.3em] text-white/70">
              {libraryName}
            </p>
            <h1 className="max-w-[240px] text-4xl font-black leading-none">
              {title}
            </h1>
          </div>

          <p className="max-w-56 text-lg text-white/80">{description}</p>
        </div>

        <div className="flex min-w-0 flex-1 flex-col gap-6 p-10">
          <div className="inline-flex w-fit items-center rounded-full bg-slate-100 px-4 py-2 text-sm font-medium text-slate-700">
            Renderer stylesheets[]
          </div>

          <div className="flex flex-col space-y-4 text-slate-950">
            <h2 className="text-4xl font-bold tracking-tight">
              Actual CSS output
            </h2>
            <p className="max-w-md text-lg leading-8 text-slate-600 whitespace-pre-wrap">
              This example collects utility candidates from the source file,
              compiles them with a CSS library, and passes the resulting
              stylesheet string to the renderer.
            </p>
          </div>

          <div className="grid grid-cols-3 gap-4 pt-4">
            <div className="flex flex-col rounded-2xl bg-cyan-50 p-5">
              <div className="text-3xl font-black text-cyan-700">1</div>
              <p className="mt-2 text-sm leading-6 text-cyan-950">
                Compile Tailwind utilities.
              </p>
            </div>

            <div className="flex flex-col rounded-2xl bg-slate-100 p-5">
              <div className="text-3xl font-black text-slate-900">2</div>
              <p className="mt-2 text-sm leading-6 text-slate-700">
                Collect utility candidates.
              </p>
            </div>

            <div className="flex flex-col rounded-2xl bg-indigo-50 p-5">
              <div className="text-3xl font-black text-indigo-700">3</div>
              <p className="mt-2 text-sm leading-6 text-indigo-950">
                Send it to <code>stylesheets</code>.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
