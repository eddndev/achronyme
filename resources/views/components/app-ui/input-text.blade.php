@props([
    'label' => '',
    'type' => 'text',
    'name',
    'id' => null,
    'placeholder' => '',
    'value' => '',
    'optional' => false,
    'optionalText' => 'Optional',
    'error' => false,
    'errorModel' => null // Alpine.js error state variable name
])

@php
    $inputId = $id ?? $name;
    // The $errorClass is now primarily for initial server-side rendering or if errorModel is not used.
    $errorClass = $error ? 'outline-red-300 focus:outline-red-600 dark:outline-red-700 dark:focus:outline-red-500' : 'outline-slate-300 focus:outline-purple-blue-600 dark:outline-slate-700 dark:focus:outline-purple-blue-500';
@endphp

<div x-data="{ inputError: {{ $errorModel ?? 'null' }} }">
    @if ($label)
        <div class="flex justify-between">
            <label for="{{ $inputId }}" class="block text-sm/6 font-medium text-slate-900 dark:text-white">{{ $label }}</label>
            @if ($optional)
                <span id="{{ $inputId }}-optional" class="text-sm/6 text-slate-500 dark:text-slate-400">{{ $optionalText }}</span>
            @endif
        </div>
    @endif
    <div class="@if($label) mt-2 @endif grid grid-cols-1">
        <input
            type="{{ $type }}"
            name="{{ $name }}"
            id="{{ $inputId }}"
            placeholder="{{ $placeholder }}"
            value="{{ $value }}"
            @if ($optional) aria-describedby="{{ $inputId }}-optional" @endif
            {{ $attributes->merge([
                'class' => 'col-start-1 row-start-1 block w-full rounded-md bg-white py-1.5 text-base pr-10 pl-3 text-slate-900 ' . $errorClass . ' placeholder:text-slate-400 focus:outline-2 focus:-outline-offset-2 sm:text-sm/6 dark:bg-white/5 dark:text-white dark:placeholder:text-slate-500'
            ]) }}
            :class="inputError ? 'outline-red-300 focus:outline-red-600 dark:outline-red-700 dark:focus:outline-red-500' : ''"
        />
        <template x-if="inputError">
            <svg viewBox="0 0 16 16" fill="currentColor" data-slot="icon" aria-hidden="true" class="pointer-events-none col-start-1 row-start-1 mr-3 size-5 self-center justify-self-end text-red-500 sm:size-4 dark:text-red-400">
            <path d="M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14ZM8 4a.75.75 0 0 1 .75.75v3a.75.75 0 0 1-1.5 0v-3A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" fill-rule="evenodd" />
            </svg>
        </template>
    </div>
    <template x-if="inputError">
        <p id="{{ $inputId }}-error" class="mt-2 text-sm text-red-600 dark:text-red-400" x-text="inputError"></p>
    </template>
</div>
