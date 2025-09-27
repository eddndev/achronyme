<x-guest-layout title="Confirm Password">
    <x-image.logo
        class="mx-auto h-80 w-auto absolute top-1/2 left-1/2 -translate-y-1/2 -translate-x-1/2 -z-10"
        src="resources/images/logo.png"
        alt="Logotipo de Achronyme"
        size="md"
        :priority="true"
    />
    <div class="border border-slate-200 bg-slate-100/75 px-6 py-12 shadow-md backdrop-blur-lg sm:rounded-lg sm:px-12 dark:border-slate-700 dark:bg-slate-900/75">
        <div class="mb-4 text-sm text-slate-800 dark:text-slate-200">
            {{ __('This is a secure area of the application. Please confirm your password before continuing.') }}
        </div>

        <form method="POST" action="{{ route('password.confirm') }}" class="space-y-6">
            @csrf

            <!-- Password -->
            <div>
                <x-input-label for="password" :value="__('Password')" />
                <div class="mt-2">
                    <x-text-input id="password" class="block mt-1 w-full"
                                    type="password"
                                    name="password"
                                    required autocomplete="current-password" />
                </div>
                <x-input-error :messages="$errors->get('password')" class="mt-2" />
            </div>

            <div class="flex justify-end">
                <x-primary-button>
                    {{ __('Confirm') }}
                </x-primary-button>
            </div>
        </form>
    </div>
</x-guest-layout>
