@php
$breadcrumbs = [
    ['name' => 'Herramientas', 'url' => route('home')],
    ['name' => 'PDS', 'url' => '#'],
    ['name' => 'Transformada de Fourier', 'url' => '#']
];
@endphp

<x-tool-layout title="Transformada de Fourier" :breadcrumbs="$breadcrumbs" icon="fx">
    <x-slot:actions>
        <x-secondary-button>
            Volver
        </x-secondary-button>
        <x-primary-button class="ml-3">
            Exportar
        </x-primary-button>
    </x-slot>

    @include('tools.fourier.ft.fourier-transform')

    <x-slot:scripts>
        @vite('resources/js/ft/app.ts')
    </x-slot>
</x-tool-layout>