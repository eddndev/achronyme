@props([
    'label' => '',
    'type' => 'text',
    'name',
    'id' => null,
    'placeholder' => '',
    'value' => '',
    'optional' => false,
    'optionalText' => 'Optional'
])

@php
    $inputId = $id ?? $name;
@endphp

<div>
    @if ($label)
        <div class="flex justify-between">
            <label for="{{ $inputId }}" class="block text-sm/6 font-medium text-slate-900 dark:text-white">{{ $label }}</label>
            @if ($optional)
                <span id="{{ $inputId }}-optional" class="text-sm/6 text-slate-500 dark:text-slate-400">{{ $optionalText }}</span>
            @endif
        </div>
    @endif
    <div class="@if($label) mt-2 @endif">
        <input
            type="{{ $type }}"
            name="{{ $name }}"
            id="{{ $inputId }}"
            placeholder="{{ $placeholder }}"
            value="{{ $value }}"
            @if ($optional) aria-describedby="{{ $inputId }}-optional" @endif
            {{ $attributes->merge([
                'class' => 'block w-full rounded-md bg-white px-3 py-1.5 text-base text-slate-900 outline-1 -outline-offset-1 outline-slate-300 placeholder:text-slate-400 focus:outline-2 focus:-outline-offset-2 focus:outline-purple-blue-600 sm:text-sm/6 dark:bg-white/5 dark:text-white dark:outline-slate-700 dark:placeholder:text-slate-500 dark:focus:outline-purple-blue-500'
            ]) }}
        />
    </div>
</div>