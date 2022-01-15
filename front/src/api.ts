import * as z from 'zod';

const root = '';

const mediaIdsSchema = z.object({
  ids: z.array(z.string()),
});

const buildUrl = (path: string) => `${root}/${path}`;

export const paths = {
  media: {
    ids: 'media/ids',
    thumb: (id: string) => `media/thumb/${id}`,
    origin: (id: string) => `media/origin/${id}`,
  },
};

export const getMediaIds = async () => {
  const path = buildUrl(paths.media.ids);
  try {
    const r = await fetch(path);
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
