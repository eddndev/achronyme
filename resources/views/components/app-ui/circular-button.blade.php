@props([
    'type' => 'button',
    'isLoading' => false,
    'icon' // The icon name, e.g., 'plus', 'trash'
])

<button {{ $attributes->merge([
    'type' => $type,
    'class' => '
        rounded-full p-2 text-white shadow-xs 
        transition-all duration-300
        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-blue-500
        dark:focus:ring-offset-slate-800
        btn-purple-blue
    '
    ]) }} 
    :disabled="{{ $isLoading }}"
>
    <div class="flex items-center justify-center">
        <template x-if="{{ $isLoading }}">
            <x-app-ui.loading-spinner class="size-5" />
        </template>
        <template x-if="!{{ $isLoading }}">
            <svg class="size-5" fill="currentColor">
                <use href="#icon-{{ $icon }}"></use>
            </svg>
        </template>
    </div>
</button>
