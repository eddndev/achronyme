@props([
    'type' => 'button',
    'icon' // The icon name, e.g., 'plus', 'trash'
])

<button {{ $attributes->merge([
    'type' => $type,
    'class' => '
        rounded-full p-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-500
        text-gray-700 dark:text-gray-300 shadow-xs
        hover:bg-gray-50 dark:hover:bg-gray-700
        focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800
        disabled:opacity-25 transition ease-in-out duration-150
    '
    ]) }}
    :disabled="isLoading"
>
    <div x-show="isLoading" class="flex items-center justify-center">
        <x-app-ui.loading-spinner class="size-5 text-gray-700 dark:text-gray-300" />
    </div>
    <svg x-show="!isLoading" class="size-5" fill="currentColor">
        <use href="#icon-{{ $icon }}"></use>
    </svg>
</button>
