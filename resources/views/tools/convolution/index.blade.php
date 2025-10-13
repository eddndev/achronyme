@php
$breadcrumbs = [
    ['name' => 'Herramientas', 'url' => route('home')],
    ['name' => 'PDS', 'url' => '#'],
    ['name' => 'Convolución', 'url' => '#']
];
@endphp

<x-tool-layout title="Convolución" :breadcrumbs="$breadcrumbs" icon="conv">
    <x-slot:actions>
        <x-app-ui.secondary-button>
            Volver
        </x-app-ui.secondary-button>
        <x-app-ui.button class="ml-3">
            Exportar
        </x-app-ui.button>
    </x-slot>

    @include('tools.convolution.convolution-content')

    <x-slot:scripts>
        @vite('resources/js/convolution/app.ts')
    </x-slot>
</x-tool-layout>