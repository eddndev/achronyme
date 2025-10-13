<x-three-column-tool>
    <x-slot:leftSidebar>
        <div class="space-y-6">
            <div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Panel de Configuración</h3>
                <div class="space-y-4">
                    <div>
                        {{-- Dimensiones --}}
                        <div class="space-y-4">
                            <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">Dimensiones del Tablero</h4>
                            <div class="grid grid-cols-2 gap-4">
                                <x-app-ui.input-text
                                    type="number"
                                    name="rows"
                                    label="Filas"
                                    x-model="rows"
                                    min="5"
                                    max="30"
                                />
                                <x-app-ui.input-text
                                    type="number"
                                    name="cols"
                                    label="Columnas"
                                    x-model="cols"
                                    min="5"
                                    max="30"
                                />
                            </div>
                            <x-app-ui.button @click="handleDimensionChange()" x-data="{ isLoading: false }">
                                Aplicar Dimensiones
                            </x-app-ui.button>
                        </div>

                        {{-- Modo de Edición --}}
                        <div class="space-y-4 mt-6">
                            <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">Modo de Edición</h4>
                            <x-app-ui.radio-list
                                name="edit_mode"
                                legend="Seleccione el modo de edición"
                                :checked-value="'set_obstacle'"
                                :options="[
                                    [
                                        'value' => 'set_start',
                                        'title' => 'Establecer Inicio',
                                        'description' => 'Define la posición inicial del agente'
                                    ],
                                    [
                                        'value' => 'set_goal',
                                        'title' => 'Añadir/Quitar Destino',
                                        'description' => 'Click para agregar o eliminar destinos'
                                    ],
                                    [
                                        'value' => 'set_obstacle',
                                        'title' => 'Añadir/Quitar Obstáculo',
                                        'description' => 'Click para agregar o eliminar obstáculos'
                                    ]
                                ]"
                                @change="setMode($event.target.value)"
                                x-model="currentMode"
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </x-slot:leftSidebar>

    {{-- Centro: Grid/Tablero --}}
    <div class="space-y-6">
        <div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Entorno del Agente</h3>
            <div id="grid-container" class="flex justify-center items-center bg-white dark:bg-gray-800 rounded-lg shadow-sm p-6">
                {{-- El JavaScript de la herramienta montará aquí el grid --}}
            </div>
        </div>
    </div>

    <x-slot:rightSidebar>
        <div class="space-y-6">
            <div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Instrucciones</h3>
                <div class="prose prose-sm dark:prose-invert text-gray-600 dark:text-gray-400">
                    <p>Configure el entorno del agente ajustando las dimensiones del tablero y colocando obstáculos.</p>
                    <ul>
                        <li>Defina el tamaño de la cuadrícula</li>
                        <li>Establezca la posición inicial del agente</li>
                        <li>Agregue obstáculos haciendo clic en las celdas</li>
                    </ul>
                </div>
                {{-- Leyenda --}}
                <div class="space-y-3 mt-6">
                    <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">Leyenda</h4>
                    <div class="grid grid-cols-2 gap-3 text-sm dark:text-gray-300">
                        <div class="flex items-center gap-2">
                            <span class="w-5 h-5 bg-green-500 rounded-sm"></span>
                            <span>Inicio</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="w-5 h-5 bg-red-500 rounded-sm"></span>
                            <span>Destino</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="w-5 h-5 bg-gray-800 dark:bg-gray-600 rounded-sm"></span>
                            <span>Obstáculo</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="w-5 h-5 bg-white dark:bg-gray-900 border-2 border-gray-300 dark:border-gray-600 rounded-sm"></span>
                            <span>Vacío</span>
                        </div>
                    </div>
                </div>
                {{-- Acciones --}}
                <div class="space-y-2 mt-6 pt-4 border-t dark:border-gray-700">
                    <x-app-ui.button @click="handleGenerateGraph()" class="w-full" x-data="{ isLoading: false }">
                        Generar Grafo
                    </x-app-ui.button>
                    <x-app-ui.secondary-button @click="handleClearObstacles()" x-data="{ isLoading: false }">
                        Limpiar Obstáculos
                    </x-app-ui.secondary-button>
                    <x-app-ui.danger-button @click="handleClear()" x-data="{ isLoading: false }">
                        Limpiar Todo
                    </x-app-ui.danger-button>
                </div>
            </div>
        </div>
    </x-slot:rightSidebar>
</x-three-column-tool>
