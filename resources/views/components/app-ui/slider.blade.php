@props([
    'label' => '',
    'name',
    'id' => null,
    'min' => 0,
    'max' => 100,
    'step' => 1,
    'value' => 0
])

@php
    $inputId = $id ?? $name;
@endphp

<div x-data="{ sliderValue: {{ $value }} }">
    <div class="flex justify-between">
        <label for="{{ $inputId }}" class="block text-sm/6 font-medium text-slate-900 dark:text-white">{{ $label }}</label>
        <span class="text-sm/6 text-slate-500 dark:text-slate-400" x-text="sliderValue"></span>
    </div>
    <div class="mt-2">
        <input
            type="range"
            name="{{ $name }}"
            id="{{ $inputId }}"
            min="{{ $min }}"
            max="{{ $max }}"
            step="{{ $step }}"
            x-model.debounce.250ms="sliderValue"
            {{ $attributes->merge([
                'class' => 'w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer dark:bg-slate-700 
                            focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-purple-blue-500 dark:focus-visible:ring-offset-slate-800 
                            [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:h-5 [&::-webkit-slider-thumb]:w-5 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-purple-blue-600 [&::-webkit-slider-thumb]:ring-2 [&::-webkit-slider-thumb]:ring-purple-blue-200 dark:[&::-webkit-slider-thumb]:bg-purple-blue-500 dark:[&::-webkit-slider-thumb]:ring-purple-blue-800
                            [&::-moz-range-thumb]:h-5 [&::-moz-range-thumb]:w-5 [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:bg-purple-blue-600 [&::-moz-range-thumb]:ring-2 [&::-moz-range-thumb]:ring-purple-blue-200 dark:[&::-moz-range-thumb]:bg-purple-blue-500 dark:[&::-moz-range-thumb]:ring-purple-blue-800'
            ]) }}
        />
    </div>
</div>