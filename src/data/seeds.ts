import seedsData from "../../shared-data/seeds.json";

export interface SeedInfo {
  id: number;
  name: string;
  level: number;
  price: number;
  exp: number;
  fruit_count: number;
  fruit_price: number;
  seasons: number;
}

/** All plantable seeds sorted by level requirement */
export const SEEDS: SeedInfo[] = seedsData;
