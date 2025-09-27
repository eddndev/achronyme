@props([
  'src' => '',                 // original_path, ej: 'resources/images/hero.jpg'
  'alt' => '',
  'class' => 'w-full h-auto',
  'sizes' => '100vw',          // puedes pasar "(min-width:1024px) 50vw, 100vw"
  'loading' => 'lazy',         // usa eager solo para LCP
  'priority' => false,         // true => eager + fetchpriority=high
  'width' => null,             // opcional (para CLS si conoces dimensiones)
  'height' => null,            // idem
  'collection' => 'default',   // por si usas otra colección
])

@php
  $srcs = get_medium_sources($src, [400,800,1200], $collection);
@endphp

<picture {{ $attributes->merge(['class' => 'block']) }}>
  <source type="image/avif" srcset="{{ $srcs['avif'] }}" sizes="{{ $sizes }}" />
  <source type="image/webp" srcset="{{ $srcs['webp'] }}" sizes="{{ $sizes }}" />
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
