@props(['leftSidebar', 'rightSidebar'])

<div class="mx-auto w-full max-w-7xl grow lg:flex xl:px-2">
    <div class="flex-1 xl:flex">

        {{-- Left sidebar --}}
        <div class="border-b border-gray-200 p-6 sm:px-6 lg:pl-8 xl:w-80 xl:shrink-0 xl:border-r xl:border-b-0 xl:pl-6 dark:border-white/10">
            <div class="xl:sticky xl:max-h-[calc(100vh-var(--sticky-top-offset,6rem))] xl:overflow-y-auto xl:py-1 xl:px-1 xl:-mx-1" style="top: var(--sticky-top-offset, 1rem);">
                {{ $leftSidebar }}
            </div>
        </div>

        {{-- Main content (center) --}}
        <div class="min-w-0 px-4 py-6 sm:px-6 lg:pl-8 xl:flex-1 xl:pl-6 overflow-visible">
            {{ $slot }}
        </div>
    </div>

    {{-- Right sidebar --}}
    <div class="shrink-0 border-t border-gray-200 p-6 sm:px-6 lg:w-80 lg:border-t-0 lg:border-l lg:pr-8 xl:pr-6 dark:border-white/10">
        <div class="lg:sticky lg:max-h-[calc(100vh-var(--sticky-top-offset,6rem))] lg:overflow-y-auto lg:py-1 lg:px-1 lg:-mx-1" style="top: var(--sticky-top-offset, 1rem);">
            {{ $rightSidebar }}
        </div>
    </div>
</div>