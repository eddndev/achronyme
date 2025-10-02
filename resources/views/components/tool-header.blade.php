@props(['title', 'icon' => null, 'actions' => null])

<div class="mt-2 md:flex md:items-center md:justify-between">
    <div class="min-w-0 flex-1 flex items-center space-x-3">
        @if ($icon)
            <div class="flex size-10 items-center justify-center rounded-lg bg-slate-100 ring-1 ring-slate-200 transition-all duration-300 group-hover:bg-purple-blue-100 group-hover:ring-purple-blue-300 dark:bg-slate-800 dark:ring-slate-700 dark:group-hover:bg-purple-blue-950 dark:group-hover:ring-purple-blue-700">
                <svg class="size-7 text-slate-500 transition-colors duration-300 group-hover:text-purple-blue-600 dark:text-slate-400 dark:group-hover:text-purple-blue-400" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                    <use href="#icon-{{ $icon }}"/>
                </svg>
            </div>
        @endif
        <h2 class="text-2xl/7 font-bold text-gray-900 sm:truncate sm:text-3xl sm:tracking-tight dark:text-white">{{ $title }}</h2>
    </div>

    @if ($actions)
        <div class="mt-4 flex shrink-0 md:mt-0 md:ml-4">
            {{ $actions }}
        </div>
    @endif
</div>
