@props(['items' => []])

<div>
    {{-- Mobile back button --}}
    <nav aria-label="Back" class="sm:hidden">
        <a href="{{ $items[0]['url'] ?? '#' }}" class="flex items-center text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300">
            <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="mr-1 -ml-1 size-5 shrink-0 text-gray-400 dark:text-gray-500">
                <path d="M11.78 5.22a.75.75 0 0 1 0 1.06L8.06 10l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z" clip-rule="evenodd" fill-rule="evenodd" />
            </svg>
            Volver
        </a>
    </nav>

    {{-- Desktop breadcrumbs --}}
    <nav aria-label="Breadcrumb" class="hidden sm:flex">
        <ol role="list" class="flex items-center space-x-4">
            @foreach ($items as $index => $item)
                <li>
                    <div class="flex {{ $index > 0 ? 'items-center' : '' }}">
                        @if ($index > 0)
                            <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-5 shrink-0 text-gray-400 dark:text-gray-500">
                                <path d="M8.22 5.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L11.94 10 8.22 6.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" fill-rule="evenodd" />
                            </svg>
                        @endif
                        <a
                            href="{{ $item['url'] ?? '#' }}"
                            @if ($loop->last) aria-current="page" @endif
                            class="{{ $index > 0 ? 'ml-4' : '' }} text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300"
                        >
                            {{ $item['name'] }}
                        </a>
                    </div>
                </li>
            @endforeach
        </ol>
    </nav>
</div>
