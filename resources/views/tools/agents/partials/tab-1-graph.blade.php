<x-three-column-tool>
    <x-slot:leftSidebar>
        <div class="space-y-6">
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Formulación Matemática</h3>
                <div class="space-y-6">
                    {{-- Título --}}
                    <div class="border-b border-slate-200 dark:border-slate-700 pb-4">
                        <p class="text-sm text-slate-600 dark:text-slate-400 mt-1">Representación formal del espacio de estados</p>
                    </div>

                    {{-- Conjunto de Estados --}}
                    <div class="space-y-2">
                        <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                            <span class="text-purple-blue-600 dark:text-purple-blue-400">S:</span> Conjunto de Estados
                        </h3>
                        <div class="bg-slate-50 dark:bg-slate-900 p-4 rounded-md border border-slate-200 dark:border-slate-700">
                            <div class="font-mono text-sm">
                                <span class="text-slate-700 dark:text-slate-300">S = {(a, b) | </span>
                                <span class="text-purple-blue-600 dark:text-purple-blue-400" x-text="`1 ≤ a ≤ ${mathFormulation.config.rows}`"></span>
                                <span class="text-slate-700 dark:text-slate-300"> ∧ </span>
                                <span class="text-purple-blue-600 dark:text-purple-blue-400" x-text="`1 ≤ b ≤ ${mathFormulation.config.cols}`"></span>
                                <template x-if="mathFormulation.obstacles.length > 0">
                                    <span>
                                        <span class="text-slate-700 dark:text-slate-300">} \ </span>
                                        <span class="text-danger" x-text="`{${mathFormulation.obstacles.map(o => `(${o.row + 1}, ${o.col + 1})`).join(', ')}}`"></span>
                                    </span>
                                </template>
                                <template x-if="mathFormulation.obstacles.length === 0">
                                    <span class="text-slate-700 dark:text-slate-300">}</span>
                                </template>
                            </div>
                            <p class="text-xs text-slate-500 dark:text-slate-400 mt-2">
                                <template x-if="mathFormulation.obstacles.length > 0">
                                    <span x-text="`Espacio de ${mathFormulation.config.rows * mathFormulation.config.cols} casillas, excluyendo ${mathFormulation.obstacles.length} obstáculo(s)`"></span>
                                </template>
                                <template x-if="mathFormulation.obstacles.length === 0">
                                    <span x-text="`Espacio de ${mathFormulation.config.rows * mathFormulation.config.cols} casillas sin obstáculos`"></span>
                                </template>
                            </p>
                        </div>
                    </div>

                    {{-- Estado Inicial --}}
                    <div class="space-y-2">
                        <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                            <span class="text-success">s₀:</span> Estado Inicial
                        </h3>
                        <div class="bg-green-50 dark:bg-green-950/20 p-4 rounded-md border border-green-200 dark:border-green-800">
                            <div class="font-mono text-sm">
                                <template x-if="mathFormulation.startPos">
                                    <span class="text-success dark:text-green-400" x-text="`s₀ = (${mathFormulation.startPos.row + 1}, ${mathFormulation.startPos.col + 1})`"></span>
                                </template>
                                <template x-if="!mathFormulation.startPos">
                                    <span class="text-slate-400 dark:text-slate-500 italic">No definido</span>
                                </template>
                            </div>
                        </div>
                    </div>

                    {{-- Estados Finales --}}
                    <div class="space-y-2">
                        <h3 class="text-lg font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                            <span class="text-danger">F:</span> Estados Finales
                        </h3>
                        <div class="bg-red-50 dark:bg-red-950/20 p-4 rounded-md border border-red-200 dark:border-red-800">
                            <div class="font-mono text-sm">
                                <template x-if="mathFormulation.goalPositions.length > 0">
                                    <span class="text-danger dark:text-red-400" x-text="`F = {${mathFormulation.goalPositions.map(g => `(${g.row + 1}, ${g.col + 1})`).join(', ')}}`"></span>
                                </template>
                                <template x-if="mathFormulation.goalPositions.length === 0">
                                    <span class="text-slate-400 dark:text-slate-500 italic">No definidos</span>
                                </template>
                            </div>
                            <template x-if="mathFormulation.goalPositions.length > 0">
                                <p class="text-xs text-slate-500 dark:text-slate-400 mt-2" x-text="`${mathFormulation.goalPositions.length} estado(s) objetivo`"></p>
                            </template>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:leftSidebar>

    {{-- Centro: Visualización del Grafo --}}
    <div class="space-y-6">
        <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Representación en Grafo</h3>
            <div class="relative bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6 w-full aspect-square min-h-[600px]">
                <div id="graph-visualization" class="w-full h-full">
                    {{-- El JavaScript de la herramienta montará aquí el grafo --}}
                </div>
                <template x-if="graphStats.nodes === 0">
                    <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                        <div class="text-center space-y-4 p-8">
                            <svg class="mx-auto h-24 w-24 text-slate-300 dark:text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"></path>
                            </svg>
                            <div class="space-y-2">
                                <p class="text-lg font-semibold text-slate-600 dark:text-slate-400">No hay grafo generado</p>
                                <p class="text-sm text-slate-500 dark:text-slate-500">Configura el entorno y presiona "Generar Grafo" para visualizarlo aquí</p>
                            </div>
                        </div>
                    </div>
                </template>
            </div>
        </div>
    </div>

    <x-slot:rightSidebar>
        <div class="space-y-6">
            {{-- Controles del Grafo --}}
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Controles</h3>
                <div class="space-y-3">
                    <x-app-ui.button
                        @click="resetGraphPositions()"
                        class="w-full"
                        x-data="{ isLoading: false }">
                        Reiniciar Posiciones
                    </x-app-ui.button>
                    <x-app-ui.secondary-button
                        @click="toggleGraphLabels()"
                        class="w-full"
                        x-data="{ isLoading: false }">
                        Alternar Etiquetas
                    </x-app-ui.secondary-button>
                    <div class="grid grid-cols-3 gap-2">
                        <x-app-ui.secondary-button
                            @click="zoomGraphIn()"
                            x-data="{ isLoading: false }">
                            <svg class="size-6 text-slate-600 dark:text-slate-400">
                                <use href="#icon-zoom-in" />
                            </svg>
                        </x-app-ui.secondary-button>
                        <x-app-ui.secondary-button
                            @click="zoomGraphOut()"
                            x-data="{ isLoading: false }">
                            <svg class="size-6 text-slate-600 dark:text-slate-400">
                                <use href="#icon-zoom-out" />
                            </svg>
                        </x-app-ui.secondary-button>
                        <x-app-ui.secondary-button
                            @click="resetGraphZoom()"
                            x-data="{ isLoading: false }">
                            Ajustar
                        </x-app-ui.secondary-button>
                    </div>
                </div>
                <template x-if="graphStats.nodes === 0">
                    <div class="mt-3 text-xs text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/20 px-3 py-2 rounded-md border border-amber-200 dark:border-amber-800">
                        <span class="font-semibold">ℹ️ Aviso:</span> Genera el grafo desde la pestaña "Configuración del Entorno" para habilitar los controles.
                    </div>
                </template>
                <template x-if="graphStats.nodes > 0">
                    <div class="mt-3 text-xs text-slate-600 dark:text-slate-400 bg-slate-50 dark:bg-slate-900 px-3 py-2 rounded-md">
                        <span class="font-semibold">💡 Tip:</span> Usa la rueda del mouse para zoom y arrastra el fondo para desplazar
                    </div>
                </template>
            </div>

            {{-- Estadísticas del Grafo --}}
            <div>
                <h3 class="text-lg font-semibold text-slate-900 dark:text-white mb-4">Estadísticas</h3>
                <div class="grid grid-cols-1 gap-3">
                    <div class="bg-purple-blue-50 dark:bg-purple-blue-950/20 p-4 rounded-md border border-purple-blue-200 dark:border-purple-blue-800 text-center">
                        <div class="text-2xl font-bold text-purple-blue-600 dark:text-purple-blue-400" x-text="graphStats.nodes">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Nodos (Estados)</div>
                    </div>
                    <div class="bg-accent-purple-50 dark:bg-accent-purple-950/20 p-4 rounded-md border border-accent-purple-200 dark:border-accent-purple-800 text-center">
                        <div class="text-2xl font-bold text-accent-purple-600 dark:text-accent-purple-400" x-text="graphStats.edges">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Aristas (Transiciones)</div>
                    </div>
                    <div class="bg-success/10 dark:bg-success/20 p-4 rounded-md border border-success/30 dark:border-success/40 text-center">
                        <div class="text-2xl font-bold text-success dark:text-green-400" x-text="graphStats.avgDegree">0</div>
                        <div class="text-xs text-slate-600 dark:text-slate-400">Grado Promedio</div>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:rightSidebar>
</x-three-column-tool>
