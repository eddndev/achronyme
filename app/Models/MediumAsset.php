<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Spatie\MediaLibrary\HasMedia;
use Spatie\MediaLibrary\InteractsWithMedia;
use Spatie\MediaLibrary\MediaCollections\Models\Media;

class MediumAsset extends Model implements HasMedia
{
    use InteractsWithMedia;

    /**
     * The attributes that aren't mass assignable.
     *
     * @var array
     */
    protected $guarded = [];

    /**
     * Registra las conversiones de imagen para este modelo.
     * Las conversiones se generan para formatos modernos (AVIF, WebP) y un fallback (JPG).
     *
     * @param \Spatie\MediaLibrary\MediaCollections\Models\Media|null $media
     * @return void
     */
    public function registerMediaConversions(?Media $media = null): void
    {
        // --- Conversiones para AVIF ---
        // Formato de última generación con la mejor compresión.
        $this->addMediaConversion('w-400-avif')->width(400)->format('avif')->nonQueued();
        $this->addMediaConversion('w-800-avif')->width(800)->format('avif')->nonQueued();
        $this->addMediaConversion('w-1200-avif')->width(1200)->format('avif')->nonQueued();

        // --- Conversiones para WEBP ---
        // Ampliamente compatible y con excelente compresión.
        $this->addMediaConversion('w-400-webp')->width(400)->format('webp')->nonQueued();
        $this->addMediaConversion('w-800-webp')->width(800)->format('webp')->nonQueued();
        $this->addMediaConversion('w-1200-webp')->width(1200)->format('webp')->nonQueued();

        // --- Conversión de Fallback ---
        // JPG para garantizar la compatibilidad con navegadores antiguos.
        // Se genera a la máxima resolución necesaria para este asset.
        $this->addMediaConversion('fallback-jpg')
             ->width(1200)
             ->format('jpg')
             ->quality(80) // Ajusta la calidad para un buen balance.
             ->nonQueued(); // NOTA: Para producción, considera eliminar nonQueued() y usar colas para no bloquear la carga.
    }
}
