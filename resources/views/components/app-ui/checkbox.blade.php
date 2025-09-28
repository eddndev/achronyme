@props([
    'name' => '',
    'id' => null,
    'value' => '',
    'checked' => false,
    'indeterminate' => false,
    'disabled' => false,
])

@php
    $inputId = $id ?? Str::uuid();
@endphp

<div
    class="group grid size-4 grid-cols-1"
    x-data="{ indeterminate: @json($indeterminate) }"
    x-init="$nextTick(() => { if ($refs.input) { $refs.input.indeterminate = indeterminate } })"
    :class="{ 'group-has-indeterminate': indeterminate }"
>
    <input
        x-ref="input"
        type="checkbox"
        name="{{ $name }}"
        id="{{ $inputId }}"
        value="{{ $value }}"
        @if($checked) checked @endif
        @if($disabled) disabled @endif
        {{ $attributes->merge(['class' => 'col-start-1 row-start-1 appearance-none rounded-sm border border-slate-300 bg-white checked:border-purple-blue-600 checked:bg-purple-blue-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-blue-600 disabled:border-slate-300 disabled:bg-slate-100 disabled:checked:bg-slate-100 dark:border-white/10 dark:bg-white/5 dark:checked:border-purple-blue-500 dark:checked:bg-purple-blue-500 dark:focus-visible:outline-purple-blue-500 dark:disabled:border-white/5 dark:disabled:bg-white/10 dark:disabled:checked:bg-white/10 forced-colors:appearance-auto']) }}
    />
    <svg viewBox="0 0 14 14" fill="none" class="pointer-events-none col-start-1 row-start-1 size-3.5 self-center justify-self-center stroke-white group-has-disabled:stroke-slate-950/25 dark:group-has-disabled:stroke-white/25">
        <path d="M3 8L6 11L11 3.5" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-0 group-has-checked:opacity-100" />
        <path d="M3 7H11" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-0 group-has-indeterminate:opacity-100" />
    </svg>
</div>
