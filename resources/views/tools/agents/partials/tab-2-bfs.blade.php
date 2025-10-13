<x-three-column-tool>
    <x-slot:leftSidebar>
        <div class="space-y-6" x-data="{ isLoading: false }">
            {{-- Información del Algoritmo --}}
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Algoritmo BFS</h3>
                <div class="prose prose-sm dark:prose-invert text-slate-600 dark:text-slate-400">
                    <p class="text-sm">Explora el grafo nivel por nivel, garantizando encontrar el camino más corto en grafos no ponderados.</p>
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
                            name="execution_mode"
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
                            id="execution-mode"
                        />
                    </div>

                    {{-- Botones: Modo Paso a Paso --}}
                    <div id="step-mode-buttons" class="space-y-3">
                        <x-app-ui.button
                            id="start-bfs-step"
                            class="w-full">
                            Iniciar BFS
                        </x-app-ui.button>

                        <div id="step-control-buttons" class="hidden w-full">
                            <x-app-ui.button-group
                                class="w-full"
                                :buttons="[
                                    ['id' => 'prev-step-bfs'],
                                    ['id' => 'next-step-bfs'],
                                    ['id' => 'reset-bfs-step']
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
                    <div id="auto-mode-buttons" class="space-y-3 hidden">
                        <x-app-ui.button
                            id="start-bfs-auto"
                            class="w-full">
                            Iniciar BFS
                        </x-app-ui.button>

                        <div id="auto-control-buttons" class="hidden w-full">
                            <x-app-ui.button-group
                                class="w-full"
                                :buttons="[
                                    ['id' => 'pause-bfs-auto'],
                                    ['id' => 'resume-bfs-auto'],
                                    ['id' => 'reset-bfs-auto']
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
                        <div id="speed-control" class="hidden">
                            <x-app-ui.slider
                                id="speed-slider"
                                name="speed"
                                label="Velocidad"
                                min="100"
                                max="2000"
                                value="1000"
                                step="100"
                            />
                            <div class="mt-2 text-center">
                                <span id="speed-value" class="text-sm font-semibold text-slate-700 dark:text-slate-300">1000 ms</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:leftSidebar>

    {{-- Centro: Visualización BFS --}}
    <div class="space-y-6">
        {{-- Árbol de Búsqueda --}}
        <div class="space-y-2">
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white">Árbol de Búsqueda BFS</h3>
            <div class="relative border-2 border-slate-300 dark:border-slate-600 rounded-lg bg-white dark:bg-slate-800 w-full h-[600px]">
                <div id="tree-visualization" class="w-full h-full overflow-auto">
                    <div class="absolute inset-0 flex items-center justify-center text-slate-400 dark:text-slate-500 pointer-events-none">
                        Árbol vacío
                    </div>
                </div>
            </div>
        </div>

        {{-- Visualización de la Cola --}}
        <div class="space-y-2" x-data="window.bfsQueueData || { queue: [] }">
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white flex items-center gap-2">
                <span class="text-purple-blue-600 dark:text-purple-blue-400">Cola (Queue):</span>
                <span class="text-sm bg-purple-blue-100 dark:bg-purple-blue-950/30 text-purple-blue-700 dark:text-purple-blue-300 px-2 py-1 rounded" x-text="`${queue.length} nodos`">0 nodos</span>
            </h3>
            <div class="flex flex-wrap items-center gap-2 min-h-[60px]">
                <div x-show="queue.length === 0" class="text-slate-400 dark:text-slate-500 text-sm">
                    La cola está vacía
                </div>
                <template x-for="(node, index) in queue" :key="node.id">
                    <div class="relative">
                        <div class="w-14 h-14 rounded-md flex items-center justify-center transition-all border-2"
                             :class="index === 0 ? 'bg-purple-blue-500 border-purple-blue-600 dark:bg-purple-blue-600 dark:border-purple-blue-500 shadow-lg scale-110' : 'bg-slate-200 dark:bg-slate-700 border-slate-300 dark:border-slate-600'">
                            <span class="font-mono text-xs font-bold transition-colors"
                                  :class="index === 0 ? 'text-white' : 'text-slate-700 dark:text-slate-200'"
                                  x-text="`(${node.position.row + 1},${node.position.col + 1})`"></span>
                        </div>
                        <div x-show="index === 0" class="absolute -top-6 left-1/2 -translate-x-1/2 text-xs font-semibold text-purple-blue-600 dark:text-purple-blue-400 whitespace-nowrap">
                            FRONT
                        </div>
                    </div>
                </template>
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
                        <div class="text-2xl font-bold text-success dark:text-green-400" id="visited-count">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Nodos Visitados</div>
                    </div>
                    <div class="bg-purple-blue-50 dark:bg-purple-blue-950/20 p-4 rounded-lg border border-purple-blue-200 dark:border-purple-blue-800 text-center">
                        <div class="text-2xl font-bold text-purple-blue-600 dark:text-purple-blue-400" id="tree-depth">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Profundidad Máxima</div>
                    </div>
                    <div class="bg-accent-purple-50 dark:bg-accent-purple-950/20 p-4 rounded-lg border border-accent-purple-200 dark:border-accent-purple-800 text-center">
                        <div class="text-2xl font-bold text-accent-purple-600 dark:text-accent-purple-400" id="path-length">-</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Longitud del Camino</div>
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
                        <span>Nodo en Cola</span>
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
