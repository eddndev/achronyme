/**
 * Tipos de casillas en el tablero
 */
export enum CellType {
  EMPTY = 'empty',
  START = 'start',
  GOAL = 'goal',
  OBSTACLE = 'obstacle'
}

/**
 * Representa una casilla en el tablero
 */
export interface Cell {
  row: number;
  col: number;
  type: CellType;
}

/**
 * Representa la posición en el tablero
 */
export interface Position {
  row: number;
  col: number;
}

/**
 * Modos de edición del tablero
 */
export enum EditMode {
  SET_START = 'set_start',
  SET_GOAL = 'set_goal',
  SET_OBSTACLE = 'set_obstacle'
}

/**
 * Configuración del entorno
 */
export interface GridConfig {
  rows: number;
  cols: number;
  cellSize: number;
}
