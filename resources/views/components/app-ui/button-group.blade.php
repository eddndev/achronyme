@props([
    'buttons' => [],
    'name' => '',
    'selected' => null,
    'type' => 'button', // 'button' o 'radio'
    'size' => 'md' // 'sm', 'md', 'lg'
])

{{--
|--------------------------------------------------------------------------
| Button Group Component
|--------------------------------------------------------------------------
| Un componente para agrupar botones visualmente conectados.
|
| Uso con botones simples:
| <x-app-ui.button-group
|     :buttons="[
|         ['id' => 'btn1', 'label' => 'Opción 1'],
|         ['id' => 'btn2', 'label' => 'Opción 2'],
|         ['id' => 'btn3', 'label' => 'Opción 3']
|     ]"
| />
|
| Uso con radio buttons:
| <x-app-ui.button-group
|     type="radio"
|     name="mode"
|     :selected="'auto'"
|     :buttons="[
|         ['value' => 'auto', 'label' => 'Automático'],
|         ['value' => 'step', 'label' => 'Manual']
|     ]"
| />
|
| Uso con slots para íconos SVG:
| <x-app-ui.button-group :buttons="[
|     ['id' => 'prev-btn'],
|     ['id' => 'next-btn']
| ]">
|     <x-slot:button-0>
|         <svg class="w-5 h-5"><use href="#icon-previous"/></svg>
|     </x-slot:button-0>
|     <x-slot:button-1>
|         <svg class="w-5 h-5"><use href="#icon-next"/></svg>
|     </x-slot:button-1>
| </x-app-ui.button-group>
|--------------------------------------------------------------------------
--}}

@php
    $sizeClasses = match($size) {
        'sm' => 'px-2 py-1.5 text-xs',
        'lg' => 'px-4 py-3 text-base',
        default => 'px-3 py-2 text-sm',
    };
@endphp

<span class="isolate inline-flex rounded-md shadow-xs dark:shadow-none">
    @if(count($buttons) > 0)
        @foreach($buttons as $index => $button)
            @php
                $isFirst = $index === 0;
                $isLast = $index === count($buttons) - 1;
                $roundedClass = $isFirst ? 'rounded-l-md' : ($isLast ? 'rounded-r-md' : '');
                $marginClass = $isFirst ? '' : '-ml-px';

                // Clases base
                $baseClasses = "relative inline-flex items-center justify-center gap-2 {$sizeClasses} font-semibold transition-colors duration-200";

                // Para radio buttons
                if ($type === 'radio') {
                    $isSelected = isset($button['value']) && $button['value'] === $selected;
                    $stateClasses = $isSelected
                        ? 'bg-purple-blue-600 text-white inset-ring-1 inset-ring-purple-blue-600 hover:bg-purple-blue-700 dark:bg-purple-blue-500 dark:inset-ring-purple-blue-500 dark:hover:bg-purple-blue-600'
                        : 'bg-white text-gray-900 inset-ring-1 inset-ring-gray-300 hover:bg-gray-50 dark:bg-white/10 dark:text-white dark:inset-ring-gray-700 dark:hover:bg-white/20';
                } else {
                    // Para botones normales
                    $stateClasses = 'bg-white text-gray-900 inset-ring-1 inset-ring-gray-300 hover:bg-gray-50 focus:z-10 dark:bg-white/10 dark:text-white dark:inset-ring-gray-700 dark:hover:bg-white/20';
                }

                $buttonClasses = "{$baseClasses} {$roundedClass} {$marginClass} {$stateClasses}";
            @endphp

            @if($type === 'radio')
                <label class="{{ $buttonClasses }} cursor-pointer">
                    <input
                        type="radio"
                        name="{{ $name }}"
                        value="{{ $button['value'] ?? '' }}"
                        @if($isSelected) checked @endif
                        class="sr-only"
                        @isset($button['id']) id="{{ $button['id'] }}" @endisset
                    />
                    @if(isset(${"button{$index}"}))
                        {{ ${"button{$index}"} }}
                    @else
                        {{ $button['label'] ?? $button['value'] ?? "Button {$index}" }}
                    @endif
                </label>
            @else
                <button
                    type="button"
                    class="{{ $buttonClasses }}"
                    @isset($button['id']) id="{{ $button['id'] }}" @endisset
                    @isset($button['disabled']) @if($button['disabled']) disabled @endif @endisset
                >
                    @if(isset(${"button{$index}"}))
                        {{ ${"button{$index}"} }}
                    @else
                        {{ $button['label'] ?? "Button {$index}" }}
                    @endif
                </button>
            @endif
        @endforeach
    @else
        {{-- Si no hay botones definidos, usar slots --}}
        @php
            $slotIndex = 0;
            $slots = [];
            while(isset(${"button{$slotIndex}"})) {
                $slots[] = $slotIndex;
                $slotIndex++;
            }
        @endphp

        @foreach($slots as $index)
            @php
                $isFirst = $index === 0;
                $isLast = $index === count($slots) - 1;
                $roundedClass = $isFirst ? 'rounded-l-md' : ($isLast ? 'rounded-r-md' : '');
                $marginClass = $isFirst ? '' : '-ml-px';
                $buttonClasses = "relative inline-flex items-center gap-2 px-3 py-2 text-sm font-semibold bg-white text-gray-900 inset-ring-1 inset-ring-gray-300 hover:bg-gray-50 focus:z-10 dark:bg-white/10 dark:text-white dark:inset-ring-gray-700 dark:hover:bg-white/20 transition-colors duration-200 {$roundedClass} {$marginClass}";
            @endphp

            <button type="button" class="{{ $buttonClasses }}">
                {{ ${"button{$index}"} }}
            </button>
        @endforeach
    @endif
</span>
