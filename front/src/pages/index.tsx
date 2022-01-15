import React, { useEffect, useState } from 'react';

const Home = (): JSX.Element => {
  const [ids, setIds] = useState([]);
  useEffect(() => {
    fetch('/media/ids').then((r) => r.json()).then((r) => setIds(r.ids));
  }, []);

  return (
    <div>
      {ids.map((id) => <span>{id}</span>)}
      hello world
    </div>
  );
};

export default Home;
