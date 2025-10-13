<x-three-column-tool>
    <x-slot:leftSidebar>
        <div class="space-y-6">
            <div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Algoritmo DFS</h3>
                <div class="prose prose-sm dark:prose-invert text-gray-600 dark:text-gray-400">
                    <p class="font-semibold text-purple-blue-600 dark:text-purple-blue-400">Depth-First Search</p>
                    <p>Explora el grafo siguiendo cada rama hasta su final antes de retroceder y explorar otras ramas.</p>
                    <div class="bg-purple-blue-50 dark:bg-purple-blue-950/20 p-4 rounded-lg mt-4">
                        <p class="text-xs font-mono">Complejidad: O(V + E)</p>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:leftSidebar>

    {{-- Centro: Visualización DFS --}}
    <div class="space-y-6">
        <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Visualización DFS</h3>
            <div id="dfs-visualization" class="bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                {{-- El JavaScript de la herramienta montará aquí la visualización DFS --}}
            </div>
        </div>
    </div>

    <x-slot:rightSidebar>
        <div class="space-y-6">
            <div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Características</h3>
                <div class="space-y-3 text-sm text-gray-600 dark:text-gray-400">
                    <div class="flex items-start space-x-2">
                        <svg class="w-5 h-5 text-purple-blue-600 dark:text-purple-blue-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                        </svg>
                        <span>Explora profundamente primero</span>
                    </div>
                    <div class="flex items-start space-x-2">
                        <svg class="w-5 h-5 text-purple-blue-600 dark:text-purple-blue-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                        </svg>
                        <span>Útil para detectar ciclos</span>
                    </div>
                    <div class="flex items-start space-x-2">
                        <svg class="w-5 h-5 text-purple-blue-600 dark:text-purple-blue-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                        </svg>
                        <span>Usa estructura de pila (LIFO)</span>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:rightSidebar>
</x-three-column-tool>