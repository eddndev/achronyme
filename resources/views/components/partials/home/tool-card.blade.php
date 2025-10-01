@props(['icon', 'title', 'description', 'url', 'target'])

<a href="{{ $url }}" class="group block rounded-lg bg-white p-6 ring-1 ring-slate-200 transition-all duration-300 hover:-translate-y-1 hover:shadow-2xl hover:ring-purple-blue-400 dark:bg-slate-900 dark:ring-slate-800 dark:hover:ring-purple-blue-500" target="{{ $target ?? '_self' }}" rel="noopener noreferrer">
    <div class="flex items-center gap-x-4">
        <div class="flex size-12 items-center justify-center rounded-lg bg-slate-100 ring-1 ring-slate-200 transition-all duration-300 group-hover:bg-purple-blue-100 group-hover:ring-purple-blue-300 dark:bg-slate-800 dark:ring-slate-700 dark:group-hover:bg-purple-blue-950 dark:group-hover:ring-purple-blue-700">
            <svg class="size-7 text-slate-500 transition-colors duration-300 group-hover:text-purple-blue-600 dark:text-slate-400 dark:group-hover:text-purple-blue-400" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                <use href="#{{ $icon }}" />
            </svg>
        </div>
        <h3 class="text-lg font-semibold text-slate-800 dark:text-slate-200">
            {{ $title }}
        </h3>
    </div>
    <p class="mt-4 text-sm text-slate-500 dark:text-slate-400">
        {{ $description }}
    </p>
</a>
