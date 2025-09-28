<x-app-layout title="Fourier Transform">

<div class="mt-6 lg:mt-8 flex min-h-full flex-col max-w-7xl mx-auto w-full">
    <div>
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
                 <div class="flex size-12 items-center justify-center rounded-lg bg-slate-100 ring-1 ring-slate-200 transition-all duration-300 group-hover:bg-purple-blue-100 group-hover:ring-purple-blue-300 dark:bg-slate-800 dark:ring-slate-700 dark:group-hover:bg-purple-blue-950 dark:group-hover:ring-purple-blue-700">
                    <svg class="size-7 text-slate-500 transition-colors duration-300 group-hover:text-purple-blue-600 dark:text-slate-400 dark:group-hover:text-purple-blue-400" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                        <use href="#icon-sf"/>
                    </svg>
                </div>
                <h2 class="text-2xl/7 font-bold text-gray-900 sm:truncate sm:text-3xl sm:tracking-tight dark:text-white">Serie de Fourier</h2>
            </div>
            <div class="mt-4 flex shrink-0 md:mt-0 md:ml-4">
            <button type="button" class="inline-flex items-center rounded-md bg-white px-3 py-2 text-sm font-semibold text-gray-900 shadow-xs inset-ring inset-ring-gray-300 hover:bg-gray-50 dark:bg-white/10 dark:text-white dark:shadow-none dark:inset-ring-white/5 dark:hover:bg-white/20">Volver</button>
            <button type="button" class="ml-3 inline-flex items-center rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white shadow-xs hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600 dark:bg-indigo-500 dark:shadow-none dark:hover:bg-indigo-400 dark:focus-visible:outline-indigo-500">Guardar</button>
            </div>
        </div>
    </div>

<!-- 3 column wrapper -->
    <div class="mx-auto w-full max-w-7xl grow lg:flex xl:px-2">
        <!-- Left sidebar & main wrapper -->
        <div class="flex-1 xl:flex">
            <div class="border-b border-gray-200 px-4 py-6 sm:px-6 lg:pl-8 xl:w-64 xl:shrink-0 xl:border-r xl:border-b-0 xl:pl-6 dark:border-white/10">
                <!-- Left column area -->
            </div>

            <div class="px-4 py-6 sm:px-6 lg:pl-8 xl:flex-1 xl:pl-6">
                <!-- Main area -->
                
            </div>
        </div>

        <div class="shrink-0 border-t border-gray-200 px-4 py-6 sm:px-6 lg:w-96 lg:border-t-0 lg:border-l lg:pr-8 xl:pr-6 dark:border-white/10">
            <!-- Right column area -->

        </div>
    </div>
</div>

</x-app-layout>