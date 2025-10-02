@props(['title', 'breadcrumbs' => [], 'icon' => null, 'scripts' => null])

<x-app-layout :title="$title">
    <div class="mt-6 lg:mt-8 flex min-h-full flex-col max-w-7xl mx-auto w-full">
        <div class="p-6 lg:px-8">
            {{-- Breadcrumbs navigation --}}
            <x-breadcrumbs :items="$breadcrumbs" />

            {{-- Tool header with title and actions --}}
            <x-tool-header :title="$title" :icon="$icon">
                <x-slot:actions>
                    {{ $actions ?? '' }}
                </x-slot>
            </x-tool-header>
        </div>

        {{-- Main content area --}}
        {{ $slot }}
    </div>

    {{-- Background decorative blur --}}
    <x-background-blur />

    {{-- Scripts section --}}
    @if ($scripts)
        @push('scripts')
            {{ $scripts }}
        @endpush
    @endif
</x-app-layout>