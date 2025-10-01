<x-app-layout title="Engineering toolbox">
<div id="hero-section" class="relative px-6 lg:px-8">
    <div class="sm:mt-32 lg:mt-16 container mx-auto text-center">
        <a href="https://github.com/eddndev/achronyme" target="_blank" class="inline-flex space-x-6 invisible sm:visible">
            <span class="rounded-lg bg-purple-50 px-3 py-1 text-sm/6 font-semibold text-purple-600 ring-1 ring-purple-600/20 ring-inset dark:bg-purple-500/10 dark:text-purple-400 dark:ring-purple-500/25">¿Qué hay de nuevo?</span>
            <span class="inline-flex items-center space-x-2 text-sm/6 font-medium text-gray-600 dark:text-gray-300">
                <span>Lanzamiento de la versión 1.0</span>
                <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-5 text-gray-400 dark:text-gray-500">
                <path d="M8.22 5.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L11.94 10 8.22 6.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" fill-rule="evenodd" />
                </svg>
            </span>
        </a>
    </div>
    <div class="mx-auto mt-8 max-w-2xl text-center">
        <h2 class="text-5xl font-semibold tracking-tight text-gray-900 sm:text-7xl dark:text-white"><span class="animated-gradient text-5xl">Achronyme</span><br>Herramientas para Ingeniería</h2>
        <p class="mt-8 text-lg font-medium text-pretty text-gray-500 sm:text-xl/8 dark:text-gray-400">Una suite de herramientas web para el análisis y visualización de conceptos clave del Procesamiento Digital de Señales (PDS), incluyendo la Transformada de Fourier, Series de Fourier, Convolución y más.</p>
    </div>
    <svg aria-hidden="true" class="absolute inset-0 -z-10 size-full mask-[radial-gradient(100%_100%_at_top_right,white,transparent)] stroke-gray-200 dark:stroke-white/10">
        <defs>
        <pattern id="983e3e4c-de6d-4c3f-8d64-b9761d1534cc" width="200" height="200" x="50%" y="-1" patternUnits="userSpaceOnUse">
            <path d="M.5 200V.5H200" fill="none" />
        </pattern>
        </defs>
        
        <rect width="100%" height="100%" fill="url(#983e3e4c-de6d-4c3f-8d64-b9761d1534cc)" stroke-width="0" />
    </svg>
    <div aria-hidden="true" class="absolute inset-x-0 -top-40 -z-10 transform-gpu overflow-hidden blur-3xl sm:-top-30">
      <div id="background-blob-1" style="clip-path: polygon(74.1% 44.1%, 100% 61.6%, 97.5% 26.9%, 85.5% 0.1%, 80.7% 2%, 72.5% 32.5%, 60.2% 62.4%, 52.4% 68.1%, 47.5% 58.3%, 45.2% 34.5%, 27.5% 76.7%, 0.1% 64.9%, 17.9% 100%, 27.6% 76.8%, 76.1% 97.7%, 74.1% 44.1%)" class="relative left-[calc(50%+3rem)] aspect-1155/678 w-144.5 -translate-x-1/2 bg-linear-to-tr from-[#ff80b5] to-[#9089fc] opacity-30 sm:left-[calc(50%+36rem)] sm:w-288.75"></div>
    </div>
    <div aria-hidden="true" class="absolute inset-x-0 -top-30 -z-10 transform-gpu overflow-hidden blur-3xl sm:-top-60">
      <div id="background-blob-2" style="clip-path: polygon(74.1% 44.1%, 100% 61.6%, 97.5% 26.9%, 85.5% 0.1%, 80.7% 2%, 72.5% 32.5%, 60.2% 62.4%, 52.4% 68.1%, 47.5% 58.3%, 45.2% 34.5%, 27.5% 76.7%, 0.1% 64.9%, 17.9% 100%, 27.6% 76.8%, 76.1% 97.7%, 74.1% 44.1%)" class="relative left-[calc(50%-11rem)] aspect-1155/678 w-144.5 -translate-x-1/2 rotate-30 bg-linear-to-tr from-[#ff80b5] to-[#9089fc] opacity-30 sm:left-[calc(50%-30rem)] sm:w-288.75"></div>
    </div>
    <x-comand.comand :tools="$tools" />
</div>


<div class="py-24 sm:py-32">

    <div class="mx-auto max-w-7xl px-6 lg:px-8">
        <div class="max-w-2xl mx-auto lg:mx-0">
            <h2 class="text-3xl font-bold tracking-tight text-gray-900 dark:text-white sm:text-4xl">Nuestras Herramientas</h2>
            <p class="mt-6 text-lg leading-8 text-gray-600 dark:text-gray-400">Explora nuestra suite de aplicaciones interactivas diseñadas para desmitificar los conceptos fundamentales del Procesamiento Digital de Señales.</p>
        </div>
    </div>

    <div class="mt-16">
        <x-partials.home.tools-grid :tools="$tools" />
    </div>
</div>

</x-app-layout>
