<x-three-column-tool>
    <x-slot:leftSidebar>
        <div class="space-y-6" x-data="{ isLoading: false }">
            {{-- Información del Algoritmo --}}
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Algoritmo DFS</h3>
                <div class="prose prose-sm dark:prose-invert text-slate-600 dark:text-slate-400">
                    <p class="text-sm">Explora el grafo siguiendo cada rama hasta su final antes de retroceder, usando una estructura de pila (LIFO).</p>
                </div>
            </div>

            {{-- Controles --}}
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Controles</h3>
                <div class="space-y-4">
                    {{-- Modo de Ejecución (Siempre visible) --}}
                    <div>
                        <h4 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-2">Modo de Ejecución</h4>
                        <x-app-ui.radio-list
                            name="execution_mode_dfs"
                            legend="Seleccione el modo de ejecución"
                            :checked-value="'step'"
                            :options="[
                                [
                                    'value' => 'step',
                                    'title' => 'Paso a Paso',
                                    'description' => 'Avanza manualmente'
                                ],
                                [
                                    'value' => 'auto',
                                    'title' => 'Automático',
                                    'description' => 'Ejecuta continuamente'
                                ]
                            ]"
                            id="execution-mode-dfs"
                        />
                    </div>

                    {{-- Número de Soluciones --}}
                    <div>
                        <h4 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-2">Configuración de Búsqueda</h4>
                        <x-app-ui.input-text
                            id="max-solutions-dfs"
                            name="max_solutions"
                            type="number"
                            label="Número de Soluciones"
                            placeholder="Ej: 1, 3, 5 o -1 para todas"
                            value="1"
                        />
                        <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">
                            Usa -1 para buscar todas las soluciones posibles
                        </p>
                    </div>

                    {{-- Botones: Modo Paso a Paso --}}
                    <div id="step-mode-buttons-dfs" class="space-y-3">
                        <x-app-ui.button
                            id="start-dfs-step"
                            class="w-full">
                            Iniciar DFS
                        </x-app-ui.button>

                        <div id="step-control-buttons-dfs" class="hidden w-full">
                            <x-app-ui.button-group
                                class="w-full"
                                :buttons="[
                                    ['id' => 'prev-step-dfs'],
                                    ['id' => 'next-step-dfs'],
                                    ['id' => 'reset-dfs-step']
                                ]">
                                <x-slot:button-0>
                                    <svg class="w-5 h-5" fill="currentColor"><use href="#icon-previous"/></svg>
                                </x-slot:button-0>
                                <x-slot:button-1>
                                    <svg class="w-5 h-5" fill="currentColor"><use href="#icon-next"/></svg>
                                </x-slot:button-1>
                                <x-slot:button-2>
                                    <svg class="w-5 h-5" fill="currentColor"><use href="#icon-restart"/></svg>
                                </x-slot:button-2>
                            </x-app-ui.button-group>
                        </div>
                    </div>

                    {{-- Botones: Modo Automático --}}
                    <div id="auto-mode-buttons-dfs" class="space-y-3 hidden">
                        <x-app-ui.button
                            id="start-dfs-auto"
                            class="w-full">
                            Iniciar DFS
                        </x-app-ui.button>

                        <div id="auto-control-buttons-dfs" class="hidden w-full">
                            <x-app-ui.button-group
                                class="w-full"
                                :buttons="[
                                    ['id' => 'pause-dfs-auto'],
                                    ['id' => 'resume-dfs-auto'],
                                    ['id' => 'reset-dfs-auto']
                                ]">
                                <x-slot:button-0>
                                    <svg class="w-5 h-5" fill="currentColor"><use href="#icon-pause"/></svg>
                                </x-slot:button-0>
                                <x-slot:button-1>
                                    <svg class="w-5 h-5" fill="currentColor"><use href="#icon-play"/></svg>
                                </x-slot:button-1>
                                <x-slot:button-2>
                                    <svg class="w-5 h-5" fill="currentColor"><use href="#icon-restart"/></svg>
                                </x-slot:button-2>
                            </x-app-ui.button-group>
                        </div>

                        {{-- Control de Velocidad (solo en modo automático) --}}
                        <div id="speed-control-dfs" class="hidden">
                            <x-app-ui.slider
                                id="speed-slider-dfs"
                                name="speed"
                                label="Velocidad"
                                min="100"
                                max="2000"
                                value="1000"
                                step="100"
                            />
                            <div class="mt-2 text-center">
                                <span id="speed-value-dfs" class="text-sm font-semibold text-slate-700 dark:text-slate-300">1000 ms</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:leftSidebar>

    {{-- Centro: Visualización DFS --}}
    <div class="space-y-6">
        {{-- Árbol de Búsqueda --}}
        <div class="space-y-2">
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white">Árbol de Búsqueda DFS</h3>
            <div class="relative border-2 border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 w-full h-[600px]">
                <div id="tree-visualization-dfs" class="w-full h-full overflow-auto">
                    <div class="absolute inset-0 flex items-center justify-center text-slate-400 dark:text-slate-500 pointer-events-none">
                        Árbol vacío
                    </div>
                </div>
            </div>
        </div>

        {{-- Visualización de la Pila --}}
        <div class="space-y-2" x-data="window.dfsStackData || { stack: [] }">
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white flex items-center gap-2">
                <span class="text-accent-purple-600 dark:text-accent-purple-400">Pila (Stack):</span>
                <span class="text-sm bg-accent-purple-100 dark:bg-accent-purple-950/30 text-accent-purple-700 dark:text-accent-purple-300 px-2 py-1 rounded" x-text="`${stack.length} nodos`">0 nodos</span>
            </h3>
            <div class="bg-accent-purple-50 dark:bg-accent-purple-950/20 p-4 rounded-lg border-2 border-accent-purple-200 dark:border-accent-purple-800 min-h-[100px] overflow-x-auto">
                <div x-show="stack.length === 0" class="text-slate-400 dark:text-slate-500 text-center text-sm">
                    La pila está vacía
                </div>
                <div x-show="stack.length > 0" class="flex flex-col gap-2">
                    <template x-for="(node, index) in stack.slice().reverse()" :key="node.id">
                        <div class="flex items-center gap-3 py-2 px-3 bg-white dark:bg-slate-700 rounded-md border-2 transition-colors"
                             :class="index === 0 ? 'border-accent-purple-500' : 'border-accent-purple-200 dark:border-accent-purple-800'">
                            <div class="w-12 h-12 rounded-full bg-accent-purple-300 dark:bg-accent-purple-600 border-2 border-accent-purple-500 flex items-center justify-center flex-shrink-0">
                                <span class="font-mono text-xs font-bold text-slate-900 dark:text-white" x-text="`(${node.position.row + 1},${node.position.col + 1})`"></span>
                            </div>
                            <div class="flex-1">
                                <div class="text-xs text-slate-600 dark:text-slate-400" x-text="`Profundidad: ${node.depth}`"></div>
                                <div x-show="index === 0" class="text-xs font-semibold text-accent-purple-600 dark:text-accent-purple-400">← Próximo a explorar (TOP)</div>
                            </div>
                        </div>
                    </template>
                </div>
            </div>
        </div>
    </div>

    <x-slot:rightSidebar>
        <div class="space-y-6">
            {{-- Estadísticas --}}
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Estadísticas</h3>
                <div class="grid grid-cols-1 gap-3">
                    <div class="bg-success/10 dark:bg-success/20 p-4 rounded-lg border border-success/30 dark:border-success/40 text-center">
                        <div class="text-2xl font-bold text-success dark:text-green-400" id="visited-count-dfs">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Nodos Visitados</div>
                    </div>
                    <div class="bg-accent-purple-50 dark:bg-accent-purple-950/20 p-4 rounded-lg border border-accent-purple-200 dark:border-accent-purple-800 text-center">
                        <div class="text-2xl font-bold text-accent-purple-600 dark:text-accent-purple-400" id="tree-depth-dfs">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Profundidad Máxima</div>
                    </div>
                    <div class="bg-purple-500/10 dark:bg-purple-500/20 p-4 rounded-lg border border-purple-500/30 dark:border-purple-500/40 text-center">
                        <div class="text-2xl font-bold text-purple-600 dark:text-purple-400" id="solutions-count-dfs">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Soluciones Encontradas</div>
                    </div>
                    <div class="bg-purple-blue-50 dark:bg-purple-blue-950/20 p-4 rounded-lg border border-purple-blue-200 dark:border-purple-blue-800 text-center">
                        <div class="text-2xl font-bold text-purple-blue-600 dark:text-purple-blue-400" id="path-length-dfs">-</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Camino Más Corto</div>
                    </div>
                </div>
            </div>

            {{-- Leyenda --}}
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Leyenda</h3>
                <div class="grid grid-cols-1 gap-3 text-xs text-slate-700 dark:text-slate-300">
                    <div class="flex items-center gap-2">
                        <div class="w-5 h-5 rounded-full bg-success border-2 border-green-600"></div>
                        <span>Nodo Inicial</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-5 h-5 rounded-full bg-danger border-2 border-red-600"></div>
                        <span>Nodo Objetivo</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-5 h-5 rounded-full bg-yellow-400 border-2 border-yellow-600"></div>
                        <span>Nodo Actual</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-5 h-5 rounded-full bg-blue-400 border-2 border-blue-600"></div>
                        <span>Nodo Visitado</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-5 h-5 rounded-full bg-slate-300 dark:bg-slate-600 border-2 border-slate-500 dark:border-slate-400"></div>
                        <span>Nodo en Pila</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-5 h-5 rounded-full bg-purple-500 border-2 border-purple-700"></div>
                        <span>Camino Solución</span>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:rightSidebar>
</x-three-column-tool>