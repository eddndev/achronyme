@props(['tabs' => []])

{{--
|--------------------------------------------------------------------------
| Tabbed Tool Wrapper Component
|--------------------------------------------------------------------------
| Un componente wrapper que permite mostrar múltiples herramientas en pestañas.
| Soporta navegación responsive y usa Alpine.js para el cambio de pestañas.
|
| Uso:
| <x-tabbed-tool :tabs="['Configuración', 'Grafo', 'BFS', 'DFS']">
|     <x-slot:tab-0>
|         <x-three-column-tool>...</x-three-column-tool>
|     </x-slot:tab-0>
|     <x-slot:tab-1>
|         <x-three-column-tool>...</x-three-column-tool>
|     </x-slot:tab-1>
|     ...
| </x-tabbed-tool>
|--------------------------------------------------------------------------
--}}

<div x-data="{ activeTab: 0 }" @change-tab.window="activeTab = $event.detail" class="w-full">

    {{-- Tab Navigation --}}
    <div class="px-4 pb-6 sm:px-6 lg:px-8">
        <div class="mx-auto max-w-7xl">

            {{-- Mobile: Select Dropdown --}}
            <div class="grid grid-cols-1 sm:hidden">
                <select
                    x-model.number="activeTab"
                    aria-label="Seleccionar pestaña"
                    class="col-start-1 row-start-1 w-full appearance-none rounded-md bg-white py-2 pr-8 pl-3 text-base text-gray-900 outline-1 -outline-offset-1 outline-gray-300 focus:outline-2 focus:-outline-offset-2 focus:outline-purple-blue-600 dark:bg-white/5 dark:text-gray-100 dark:outline-white/10 dark:*:bg-gray-800 dark:focus:outline-purple-blue-500 transition-colors duration-200"
                >
                    @foreach($tabs as $index => $tabTitle)
                        <option value="{{ $index }}">{{ $tabTitle }}</option>
                    @endforeach
                </select>

                {{-- Dropdown icon --}}
                <svg viewBox="0 0 16 16" fill="currentColor" data-slot="icon" aria-hidden="true"
                     class="pointer-events-none col-start-1 row-start-1 mr-2 size-5 self-center justify-self-end fill-gray-500 dark:fill-gray-400">
                    <path d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" fill-rule="evenodd" />
                </svg>
            </div>

            {{-- Desktop: Tab Navigation --}}
            <div class="hidden sm:block">
                <nav class="flex border-b border-gray-200 py-4 dark:border-white/10" aria-label="Tabs">
                    <ul role="list" class="flex min-w-full flex-none gap-x-8 px-2 text-sm/6 font-semibold">
                        @foreach($tabs as $index => $tabTitle)
                            <li>
                                <button
                                    @click="activeTab = {{ $index }}"
                                    :class="activeTab === {{ $index }}
                                        ? 'text-purple-blue-600 dark:text-purple-blue-400 border-b-2 border-purple-blue-600 dark:border-purple-blue-400'
                                        : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-white border-b-2 border-transparent hover:border-gray-300 dark:hover:border-gray-600'"
                                    class="pb-2 transition-all duration-200 ease-in-out"
                                    type="button"
                                    role="tab"
                                    :aria-selected="activeTab === {{ $index }}"
                                    aria-controls="tab-panel-{{ $index }}"
                                >
                                    {{ $tabTitle }}
                                </button>
                            </li>
                        @endforeach
                    </ul>
                </nav>
            </div>
        </div>
    </div>

    {{-- Tab Panels --}}
    <div class="relative">
        @foreach($tabs as $index => $tabTitle)
            <div
                x-show="activeTab === {{ $index }}"
                x-transition:enter="transition ease-out duration-300"
                x-transition:enter-start="opacity-0 transform translate-x-4"
                x-transition:enter-end="opacity-100 transform translate-x-0"
                x-transition:leave="transition ease-in duration-200"
                x-transition:leave-start="opacity-100 transform translate-x-0"
                x-transition:leave-end="opacity-0 transform -translate-x-4"
                role="tabpanel"
                id="tab-panel-{{ $index }}"
                :aria-hidden="activeTab !== {{ $index }}"
                class="focus:outline-none"
            >
                @if(isset(${"tab{$index}"}))
                    {{ ${"tab{$index}"} }}
                @else
                    <div class="p-8 text-center text-gray-500 dark:text-gray-400">
                        <p class="text-lg">Contenido de la pestaña "{{ $tabTitle }}" no disponible.</p>
                        <p class="text-sm mt-2">Asegúrate de definir el slot <code class="bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">tab-{{ $index }}</code></p>
                    </div>
                @endif
            </div>
        @endforeach
    </div>
</div>
