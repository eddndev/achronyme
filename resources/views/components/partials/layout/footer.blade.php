<footer class="bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-800">
    <div class="mx-auto max-w-7xl px-6 py-12 lg:px-8">
        <div class="flex flex-col items-center justify-center gap-y-4">
            <x-image.logo class="w-auto h-12" src="resources/images/logo.png"/>
            <h2 class="text-2xl font-semibold text-gray-900 dark:text-white">ACHRONYME</h2>
            <p class="max-w-2xl text-center text-sm/6 text-balance text-gray-600 dark:text-gray-400">
                Una suite de herramientas web para el análisis y visualización de conceptos clave del Procesamiento Digital de Señales (PDS).
            </p>
        </div>

        <div class="mt-10 flex justify-center gap-x-6">
            <a href="https://eddndev.com" target="_blank" rel="noopener noreferrer" class="text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-300">
                <span class="sr-only">Portafolio</span>
                <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path></svg>
            </a>
            <a href="https://github.com/eddndev/achronyme" target="_blank" rel="noopener noreferrer" class="text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-300">
                <span class="sr-only">GitHub</span>
                <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" class="size-6">
                    <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd" fill-rule="evenodd" />
                </svg>
            </a>
            <a href="mailto:contacto@eddndev.com" class="text-gray-600 hover:text-gray-800 dark:text-gray-400 dark:hover:text-gray-300">
                <span class="sr-only">Email</span>
                <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"></path></svg>
            </a>
        </div>

        <div class="mt-10 border-t border-gray-900/10 pt-8 dark:border-white/10">
            <p class="text-center text-sm/6 text-gray-600 dark:text-gray-400">
                &copy; {{ date('Y') }} Achronyme. Un proyecto de código abierto bajo la <a href="https://opensource.org/licenses/MIT" target="_blank" rel="noopener noreferrer" class="hover:underline">licencia MIT</a>.
            </p>
            <p class="mt-2 text-center text-xs text-gray-500 dark:text-gray-400">
                Desarrollado con ❤️ por <a href="https://eddndev.com" target="_blank" rel="noopener noreferrer" class="font-semibold text-indigo-600 dark:text-indigo-400 hover:underline">Eduardo Alonso</a> y el Equipo 1 de PDS.
            </p>
        </div>
    </div>
</footer>
