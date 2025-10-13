import { Cell, CellType, Position, GridConfig } from './types';

/**
 * Clase que maneja el estado del tablero del agente
 */
export class Grid {
  private cells: Cell[][];
  private config: GridConfig;
  private startPosition: Position | null = null;
  private goalPositions: Position[] = [];

  constructor(config: GridConfig) {
    this.config = config;
    this.cells = this.initializeCells();
  }

  /**
   * Inicializa todas las casillas como vacías
   */
  private initializeCells(): Cell[][] {
    const cells: Cell[][] = [];
    for (let row = 0; row < this.config.rows; row++) {
      cells[row] = [];
      for (let col = 0; col < this.config.cols; col++) {
        cells[row][col] = {
          row,
          col,
          type: CellType.EMPTY
        };
      }
    }
    return cells;
  }

  /**
   * Obtiene una casilla específica
   */
  getCell(row: number, col: number): Cell | null {
    if (this.isValidPosition(row, col)) {
      return this.cells[row][col];
    }
    return null;
  }

  /**
   * Obtiene todas las casillas
   */
  getAllCells(): Cell[][] {
    return this.cells;
  }

  /**
   * Obtiene todas las casillas en formato plano
   */
  getFlatCells(): Cell[] {
    return this.cells.flat();
  }

  /**
   * Verifica si una posición es válida
   */
  private isValidPosition(row: number, col: number): boolean {
    return row >= 0 && row < this.config.rows && col >= 0 && col < this.config.cols;
  }

  /**
   * Establece la posición de inicio
   */
  setStart(row: number, col: number): boolean {
    if (!this.isValidPosition(row, col)) return false;

    // Limpiar posición de inicio anterior
    if (this.startPosition) {
      this.cells[this.startPosition.row][this.startPosition.col].type = CellType.EMPTY;
    }

    // Establecer nueva posición de inicio
    this.cells[row][col].type = CellType.START;
    this.startPosition = { row, col };
    return true;
  }

  /**
   * Añade una posición de destino
   */
  addGoal(row: number, col: number): boolean {
    if (!this.isValidPosition(row, col)) return false;

    // Verificar si ya es un destino
    if (this.goalPositions.some(pos => pos.row === row && pos.col === col)) {
      return false;
    }

    this.cells[row][col].type = CellType.GOAL;
    this.goalPositions.push({ row, col });
    return true;
  }

  /**
   * Elimina una posición de destino
   */
  removeGoal(row: number, col: number): boolean {
    const index = this.goalPositions.findIndex(pos => pos.row === row && pos.col === col);
    if (index !== -1) {
      this.goalPositions.splice(index, 1);
      this.cells[row][col].type = CellType.EMPTY;
      return true;
    }
    return false;
  }

  /**
   * Establece un obstáculo
   */
  setObstacle(row: number, col: number): boolean {
    if (!this.isValidPosition(row, col)) return false;

    const cell = this.cells[row][col];

    // No permitir obstáculos en inicio o destino
    if (cell.type === CellType.START || cell.type === CellType.GOAL) {
      return false;
    }

    cell.type = CellType.OBSTACLE;
    return true;
  }

  /**
   * Elimina un obstáculo
   */
  removeObstacle(row: number, col: number): boolean {
    if (!this.isValidPosition(row, col)) return false;

    const cell = this.cells[row][col];
    if (cell.type === CellType.OBSTACLE) {
      cell.type = CellType.EMPTY;
      return true;
    }
    return false;
  }

  /**
   * Limpia toda la configuración
   */
  clear(): void {
    this.cells = this.initializeCells();
    this.startPosition = null;
    this.goalPositions = [];
  }

  /**
   * Limpia solo los obstáculos
   */
  clearObstacles(): void {
    for (let row = 0; row < this.config.rows; row++) {
      for (let col = 0; col < this.config.cols; col++) {
        if (this.cells[row][col].type === CellType.OBSTACLE) {
          this.cells[row][col].type = CellType.EMPTY;
        }
      }
    }
  }

  /**
   * Redimensiona el tablero
   */
  resize(rows: number, cols: number): void {
    this.config.rows = rows;
    this.config.cols = cols;
    this.cells = this.initializeCells();
    this.startPosition = null;
    this.goalPositions = [];
  }

  /**
   * Obtiene la configuración actual
   */
  getConfig(): GridConfig {
    return { ...this.config };
  }

  /**
   * Obtiene la posición de inicio
   */
  getStartPosition(): Position | null {
    return this.startPosition ? { ...this.startPosition } : null;
  }

  /**
   * Obtiene las posiciones de destino
   */
  getGoalPositions(): Position[] {
    return [...this.goalPositions];
  }

  /**
   * Verifica si el tablero está listo para ejecutar un algoritmo
   */
  isReady(): boolean {
    return this.startPosition !== null && this.goalPositions.length > 0;
  }
}
