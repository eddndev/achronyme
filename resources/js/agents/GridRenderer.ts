import { Grid } from './Grid';
import { Cell, CellType } from './types';

/**
 * Evento de click en una casilla
 */
export interface CellClickEvent {
  row: number;
  col: number;
  cell: Cell;
}

/**
 * Renderizador visual del tablero usando HTML/CSS Grid
 */
export class GridRenderer {
  private container: HTMLElement;
  private gridElement: HTMLElement | null = null;
  private grid: Grid;
  private cellSize: number = 50;
  private onCellClick: ((event: CellClickEvent) => void) | null = null;

  constructor(containerId: string, grid: Grid) {
    const container = document.getElementById(containerId);
    if (!container) {
      throw new Error(`Container with id "${containerId}" not found`);
    }
    this.container = container;
    this.grid = grid;
  }

  /**
   * Establece el callback para clicks en casillas
   */
  setCellClickHandler(handler: (event: CellClickEvent) => void): void {
    this.onCellClick = handler;
  }

  /**
   * Renderiza el tablero
   */
  render(): void {
    // Limpiar completamente el contenedor
    this.container.innerHTML = '';

    const config = this.grid.getConfig();
    const cells = this.grid.getFlatCells();

    // Crear el contenedor del grid
    this.gridElement = document.createElement('div');
    this.gridElement.className = 'inline-grid gap-0 border-2 border-slate-300 dark:border-slate-600 rounded-lg overflow-hidden shadow-lg bg-slate-100 dark:bg-slate-900';
    this.gridElement.style.gridTemplateColumns = `repeat(${config.cols}, ${this.cellSize}px)`;
    this.gridElement.style.gridTemplateRows = `repeat(${config.rows}, ${this.cellSize}px)`;

    // Crear las celdas
    cells.forEach(cell => {
      const cellElement = this.createCellElement(cell);
      this.gridElement!.appendChild(cellElement);
    });

    this.container.appendChild(this.gridElement);
  }

  /**
   * Crea un elemento HTML para una celda
   */
  private createCellElement(cell: Cell): HTMLElement {
    const cellElement = document.createElement('div');
    cellElement.className = this.getCellClasses(cell.type);
    cellElement.dataset.row = cell.row.toString();
    cellElement.dataset.col = cell.col.toString();

    // Agregar contenido: coordenadas
    const label = document.createElement('span');
    label.className = 'text-xs font-mono select-none pointer-events-none';
    label.textContent = `${cell.row + 1},${cell.col + 1}`;
    cellElement.appendChild(label);

    // Event listeners
    cellElement.addEventListener('click', () => {
      if (this.onCellClick) {
        this.onCellClick({ row: cell.row, col: cell.col, cell });
      }
    });

    return cellElement;
  }

  /**
   * Obtiene las clases CSS para una celda según su tipo
   */
  private getCellClasses(type: CellType): string {
    const baseClasses = 'flex items-center justify-center border border-slate-300 dark:border-slate-800 cursor-pointer transition-all duration-150 hover:scale-105 hover:shadow-md';

    switch (type) {
      case CellType.START:
        return `${baseClasses} bg-success text-white font-bold`;
      case CellType.GOAL:
        return `${baseClasses} bg-danger text-white font-bold`;
      case CellType.OBSTACLE:
        return `${baseClasses} bg-slate-800 dark:bg-slate-900 text-slate-50`;
      case CellType.EMPTY:
      default:
        return `${baseClasses} bg-slate-50 dark:bg-slate-700 text-slate-500 dark:text-slate-400`;
    }
  }

  /**
   * Actualiza el renderizado del tablero
   */
  update(): void {
    if (!this.gridElement) {
      this.render();
      return;
    }

    const cells = this.grid.getFlatCells();
    const cellElements = this.gridElement.children;

    // Actualizar las clases de cada celda
    cells.forEach((cell, index) => {
      const cellElement = cellElements[index] as HTMLElement;
      if (cellElement) {
        cellElement.className = this.getCellClasses(cell.type);
        cellElement.dataset.row = cell.row.toString();
        cellElement.dataset.col = cell.col.toString();
      }
    });
  }

  /**
   * Ajusta el tamaño de las celdas dinámicamente
   */
  setCellSize(size: number): void {
    this.cellSize = size;
    this.render();
  }

  /**
   * Limpia el renderizado
   */
  clear(): void {
    this.container.innerHTML = '';
    this.gridElement = null;
  }
}
