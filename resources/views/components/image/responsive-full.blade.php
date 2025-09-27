@props([
  'src' => '',                 {{-- original_path, ej: 'resources/images/hero-hq.jpg' --}}
  'alt' => '',
  'class' => 'w-full h-auto',
  'sizes' => '100vw',          {{-- ej: "(min-width:1280px) 1280px, 100vw" --}}
  'loading' => 'lazy',         {{-- usa eager sólo si es LCP --}}
  'priority' => false,         {{-- true => eager + fetchpriority="high" --}}
  'width' => null,             {{-- para evitar CLS si conoces dimensiones --}}
  'height' => null,
  'collection' => 'default',
  'widths' => [1600,2400,3200],{{-- limitadas a las conversiones disponibles --}}
])

@php
  // Si tienes los helpers, úsalos; si no, fallback simple.
  $hasHelper = function_exists('get_full_sources');
  $srcs = $hasHelper
    ? get_full_sources($src, $widths, $collection)
    : [
        'avif'     => '',
        'webp'     => '',
        'fallback' => asset('images/placeholder-large.jpg'),
      ];
@endphp

<picture {{ $attributes->merge(['class' => 'block']) }}>
  @if(!empty($srcs['avif']))
    <source type="image/avif" srcset="{{ $srcs['avif'] }}" sizes="{{ $sizes }}" />
  @endif

  @if(!empty($srcs['webp']))
    <source type="image/webp" srcset="{{ $srcs['webp'] }}" sizes="{{ $sizes }}" />
  @endif

  <img
    src="{{ $srcs['fallback'] }}"
    alt="{{ $alt }}"
    @if($width)  width="{{ $width }}"   @endif
    @if($height) height="{{ $height }}" @endif
    class="{{ $class }}"
    loading="{{ $priority ? 'eager' : $loading }}"
    decoding="async"
    @if($priority) fetchpriority="high" @endif
  />
</picture>
