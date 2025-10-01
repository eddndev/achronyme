<x-app-layout title="Fourier Transform">

<div class="mt-6 lg:mt-8 flex min-h-full flex-col max-w-7xl mx-auto w-full">
    <div class="p-6 lg:px-8">
        <div>
            <nav aria-label="Back" class="sm:hidden">
                <a href="#" class="flex items-center text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300">
                    <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="mr-1 -ml-1 size-5 shrink-0 text-gray-400 dark:text-gray-500">
                        <path d="M11.78 5.22a.75.75 0 0 1 0 1.06L8.06 10l3.72 3.72a.75.75 0 1 1-1.06 1.06l-4.25-4.25a.75.75 0 0 1 0-1.06l4.25-4.25a.75.75 0 0 1 1.06 0Z" clip-rule="evenodd" fill-rule="evenodd" />
                    </svg>
                Volver
                </a>
            </nav>
            <nav aria-label="Breadcrumb" class="hidden sm:flex">
                <ol role="list" class="flex items-center space-x-4">
                    <li>
                    <div class="flex">
                        <a href="#" class="text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300">Herramientas</a>
                    </div>
                </li>
                <li>
                    <div class="flex items-center">
                        <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-5 shrink-0 text-gray-400 dark:text-gray-500">
                        <path d="M8.22 5.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L11.94 10 8.22 6.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" fill-rule="evenodd" />
                        </svg>
                        <a href="#" class="ml-4 text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300">PDS</a>
                    </div>
                </li>
                <li>
                <div class="flex items-center">
                    <svg viewBox="0 0 20 20" fill="currentColor" data-slot="icon" aria-hidden="true" class="size-5 shrink-0 text-gray-400 dark:text-gray-500">
                    <path d="M8.22 5.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L11.94 10 8.22 6.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" fill-rule="evenodd" />
                    </svg>
                    <a href="#" aria-current="page" class="ml-4 text-sm font-medium text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300">Serie de Fourier</a>
                </div>
                </li>
            </ol>
            </nav>
        </div>
        <div class="mt-2 md:flex md:items-center md:justify-between">
            <div class="min-w-0 flex-1 flex items-center space-x-3">
                 <div class="flex size-10 items-center justify-center rounded-lg bg-slate-100 ring-1 ring-slate-200 transition-all duration-300 group-hover:bg-purple-blue-100 group-hover:ring-purple-blue-300 dark:bg-slate-800 dark:ring-slate-700 dark:group-hover:bg-purple-blue-950 dark:group-hover:ring-purple-blue-700">
                    <svg class="size-7 text-slate-500 transition-colors duration-300 group-hover:text-purple-blue-600 dark:text-slate-400 dark:group-hover:text-purple-blue-400" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                        <use href="#icon-sf"/>
                    </svg>
                </div>
                <h2 class="text-2xl/7 font-bold text-gray-900 sm:truncate sm:text-3xl sm:tracking-tight dark:text-white">Serie de Fourier</h2>
            </div>
            <div class="mt-4 flex shrink-0 md:mt-0 md:ml-4">
            <x-secondary-button>
                Volver
            </x-secondary-button>
            <x-primary-button class="ml-3">
                Guardar
            </x-primary-button>
            
            </div>
        </div>
    </div>

    @include('tools.fourier.fs.fourier-series')
</div>

@push('scripts')
    @vite('resources/js/fs/app.ts')
@endpush

</x-app-layout>