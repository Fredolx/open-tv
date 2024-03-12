import { Channel } from './channel';
import { Xtream } from './xtream';

export class Source {
  name!: string;
  channels!: Array<Channel>;
  xtream?: Xtream;
  url?: string;
  favs!: Array<Channel>;
}
