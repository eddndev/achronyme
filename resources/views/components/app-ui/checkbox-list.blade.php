@props([
    'legend' => '',
    'options' => [],
    'checkedValues' => []
])

<fieldset class="border-t border-b border-slate-200 dark:border-white/10">
    @if($legend)
        <legend class="sr-only">{{ $legend }}</legend>
    @endif
    <div class="divide-y divide-slate-200 dark:divide-white/10">
        @foreach ($options as $option)
            @php
                $name = $option['name'] ?? 'checkbox_'.Str::slug($option['title']);
                $title = $option['title'] ?? '';
                $description = $option['description'] ?? '';
                $value = $option['value'] ?? '1';
                $id = 'checkbox_'.Str::slug($name);
                $isChecked = in_array($value, $checkedValues);
            @endphp
            <div class="relative flex gap-3 pt-3.5 pb-4">
                <div class="min-w-0 flex-1 text-sm/6">
                    <label for="{{ $id }}" class="cursor-pointer font-medium text-slate-900 dark:text-white">{{ $title }}</label>
                    @if($description)
                        <p id="{{ $id }}-description" class="text-slate-500 dark:text-slate-400">{{ $description }}</p>
                    @endif
                </div>
                <div class="flex h-6 shrink-0 items-center">
                    <x-app-ui.checkbox
                        :name="$name"
                        :id="$id"
                        :value="$value"
                        :checked="$isChecked"
                        :aria-describedby="$description ? $id.'-description' : null"
                    />
                </div>
            </div>
        @endforeach
    </div>
</fieldset>