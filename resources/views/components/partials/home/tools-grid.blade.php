@php
$tools = [
    ['title' => 'Transformada de Fourier', 'description' => 'Calcular el espectro de magnitud y fase de una señal.', 'url' => '#', 'icon' => 'icon-fx'],
    ['title' => 'Serie de Fourier', 'description' => 'Analizar funciones periódicas descomponiéndolas en senos y cosenos.', 'url' => '#', 'icon' => 'icon-sf'],
    ['title' => 'Convolución', 'description' => 'Permite visualizar la convolución de dos señales en tiempo real.', 'url' => '#', 'icon' => 'icon-conv'],
    ['title' => 'GitHub', 'description' => 'Explora el código fuente, la documentación y contribuye al proyecto.', 'url' => '#', 'icon' => 'icon-github'],
];
@endphp

<div class="mx-auto max-w-6xl px-4 sm:px-6 lg:px-8">
    <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        @foreach ($tools as $tool)
            <x-partials.home.tool-card
                :icon="$tool['icon']"
                :title="$tool['title']"
                :description="$tool['description']"
                :url="$tool['url']"
            />
        @endforeach
    </div>
</div>
