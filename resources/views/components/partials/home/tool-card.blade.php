@props(['icon', 'title', 'description', 'url', 'target'])

<a href="{{ $url }}" class="group relative block overflow-hidden rounded-lg bg-white/40 p-6 shadow-lg backdrop-blur-xl backdrop-saturate-150 ring-1 ring-white/20 transition-all duration-300 hover:-translate-y-1 hover:bg-white/60 hover:shadow-2xl hover:shadow-purple-blue-500/30 hover:ring-white/40 dark:bg-slate-900/40 dark:ring-slate-700/30 dark:hover:bg-slate-800/60 dark:hover:shadow-purple-blue-400/20" target="{{ $target ?? '_self' }}" rel="noopener noreferrer">
    <!-- Gradiente de fondo animado -->
    <div class="absolute inset-0 -z-10 bg-gradient-to-br from-purple-blue-400/10 via-transparent to-purple-blue-600/10 opacity-0 transition-opacity duration-500 group-hover:opacity-100"></div>

    <!-- Efecto de luz líquida -->
    <div class="absolute -inset-1 -z-20 bg-gradient-to-r from-purple-blue-500/20 via-purple-blue-400/20 to-purple-blue-600/20 opacity-0 blur-xl transition-opacity duration-500 group-hover:opacity-100"></div>

    <div class="flex items-center gap-x-4">
        <div class="flex size-12 items-center justify-center rounded-md bg-gradient-to-br from-white/80 to-white/40 shadow-lg backdrop-blur-sm ring-1 ring-white/30 transition-all duration-500 group-hover:scale-110 group-hover:rotate-6 group-hover:from-purple-blue-400/90 group-hover:to-purple-blue-600/90 group-hover:shadow-xl group-hover:shadow-purple-blue-500/50 group-hover:ring-purple-blue-300/50 dark:from-slate-700/60 dark:to-slate-800/40 dark:ring-slate-600/30 dark:group-hover:from-purple-blue-500/80 dark:group-hover:to-purple-blue-700/80">
            <svg class="size-7 text-slate-600 transition-all duration-500 group-hover:scale-110 group-hover:text-white group-hover:drop-shadow-lg dark:text-slate-300 dark:group-hover:text-white" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                <use href="#{{ $icon }}" />
            </svg>
        </div>
        <h3 class="text-lg font-semibold text-slate-900 transition-all duration-300 group-hover:text-purple-blue-600 dark:text-slate-100 dark:group-hover:text-purple-blue-400">
            {{ $title }}
        </h3>
    </div>
    <p class="mt-4 text-sm text-slate-700 transition-colors duration-300 group-hover:text-slate-900 dark:text-slate-300 dark:group-hover:text-slate-200">
        {{ $description }}
    </p>
</a>
