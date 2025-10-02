@php
$breadcrumbs = [
    ['name' => 'Herramientas', 'url' => route('home')],
    ['name' => 'PDS', 'url' => '#'],
    ['name' => 'Serie de Fourier', 'url' => '#']
];
@endphp

<x-tool-layout title="Serie de Fourier" :breadcrumbs="$breadcrumbs" icon="sf">
    <x-slot:actions>
        <x-secondary-button>
            Volver
        </x-secondary-button>
        <x-primary-button class="ml-3">
            Guardar
        </x-primary-button>
    </x-slot>

    @include('tools.fourier.fs.fourier-series')

    <x-slot:scripts>
        @vite('resources/js/fs/app.ts')
    </x-slot>
</x-tool-layout>