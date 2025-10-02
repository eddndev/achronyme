@php
$breadcrumbs = [
    ['name' => 'Herramientas', 'url' => route('home')],
    ['name' => 'PDS', 'url' => '#'],
    ['name' => 'Convolución', 'url' => '#']
];
@endphp

<x-tool-layout title="Convolución" :breadcrumbs="$breadcrumbs" icon="conv">
    <x-slot:actions>
        <x-secondary-button>
            Volver
        </x-secondary-button>
        <x-primary-button class="ml-3">
            Exportar
        </x-primary-button>
    </x-slot>

    @include('tools.convolution.convolution-content')

    <x-slot:scripts>
        @vite('resources/js/convolution/app.ts')
    </x-slot>
</x-tool-layout>