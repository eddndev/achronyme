@props([
    'type' => 'button',
    'isLoading' => false,
    'loadingText' => 'Cargando...'
])

<button {{ $attributes->merge([
    'type' => $type,
    'class' => 'inline-flex items-center justify-center px-4 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-500 rounded-md font-bold text-md text-gray-700 dark:text-gray-300 shadow-xs hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-hidden focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 dark:focus:ring-offset-gray-800 disabled:opacity-25 transition ease-in-out duration-150'
    ]) }} 
    :disabled="{{ $isLoading }}"
>
    <div class="flex items-center justify-center">
        <template x-if="{{ $isLoading }}">
            <div class="flex items-center justify-center">
                <x-app-ui.loading-spinner class="h-5 w-5 mr-2 text-gray-700 dark:text-gray-300" />
                <span x-text="'{{ $loadingText }}'"></span>
            </div>
        </template>
        <template x-if="!{{ $isLoading }}">
            <span>{{ $slot }}</span>
        </template>
    </div>
</button>
