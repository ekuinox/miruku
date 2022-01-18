import React, { useEffect, useState } from 'react';
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
  const [ids, setIds] = useState<ReadonlyArray<string> | null>(null);
  useEffect(() => {
    getMediaIds().then((mediaIds) => {
      if (mediaIds == null) {
        return;
      }
      setIds(mediaIds.ids ?? []);
    });
  }, []);

  if (ids == null) {
    return (
      <span>
        null
      </span>
    );
  }

  return (
    <Images ids={ids} />
  );
};

export default Home;
