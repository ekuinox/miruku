import React, { useCallback, useEffect, useRef, useState } from 'react';
import ImageList from '@mui/material/ImageList';
import ImageListItem from '@mui/material/ImageListItem';
import { LazyLoadImage } from 'react-lazy-load-image-component';
import { getMediaIds } from '../api';

const Image = ({ id }: { id: string }): JSX.Element => {
  return (
    <ImageListItem key={id}>
      <a href={`/media/origin/${id}`}>
        <LazyLoadImage
          height={150}
          src={`/media/thumb/${id}`}
          alt={id}
          loading="lazy"
        />
      </a>
    </ImageListItem>
  );
};

const Images = ({ ids }: { ids: ReadonlyArray<string> }): JSX.Element => {
  return (
    <ImageList cols={2}>
      {ids.map((id) => <Image key={id} id={id} />)}
    </ImageList>
  );
};

const Home = (): JSX.Element => {
  const [ids, setIds] = useState<ReadonlyArray<string>>([]);
  const next = useRef<number | null>(null);

  const loadNext = useCallback(() => {
    getMediaIds(next.current).then((mediaIds) => {
      if (mediaIds == null) {
        return;
      }
      next.current = mediaIds.last;
      const prev = ids ?? [];
      const ids_ = (mediaIds.ids ?? []).filter((id) => !prev.includes(id));
      setIds([...prev, ...ids_]);
    });
  }, [ids]);

  useEffect(() => {
    loadNext();
  }, []);

  return (
    <div>
      <Images ids={ids} />
      <button onClick={loadNext}>
        次を読み込む
      </button>
    </div>
  );
};

export default Home;
