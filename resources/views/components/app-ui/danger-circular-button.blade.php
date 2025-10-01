@props([
    'type' => 'button',
    'icon' // The icon name, e.g., 'plus', 'trash'
])

<button {{ $attributes->merge([
    'type' => $type,
    'class' => '
        rounded-full p-2 text-white shadow-xs
        transition-all duration-300
        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500
        dark:focus:ring-offset-slate-800
        btn-danger
    '
    ]) }}
    :disabled="isLoading"
>
    <div x-show="isLoading" class="flex items-center justify-center">
        <x-app-ui.loading-spinner class="size-5" />
    </div>
    <svg x-show="!isLoading" class="size-5" fill="currentColor">
        <use href="#icon-{{ $icon }}"></use>
    </svg>
</button>