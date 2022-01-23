import * as z from 'zod';

const root = '';

const mediaIdsSchema = z.object({
  ids: z.array(z.string()),
  last: z.number(),
});

const buildUrl = (path: string) => `${root}/${path}`;

export const paths = {
  media: {
    ids: 'media/ids',
    thumb: (id: string) => `media/thumb/${id}`,
    origin: (id: string) => `media/origin/${id}`,
  },
};

export const getMediaIds = async (begin: number | null, count = 100) => {
  // クエリストリング作るところなんとかせえ
  const path = buildUrl(paths.media.ids) + '?count=' + count;
  try {
    const path_ = begin != null ? `${path}&begin=${begin}` : path;
    const r = await fetch(path_);
    const data = await r.json();
    const parsed = mediaIdsSchema.safeParse(data);
    if (parsed.success) {
      return parsed.data;
    }
    return null;
  } catch (e: unknown) {
    return null;
  }
};
