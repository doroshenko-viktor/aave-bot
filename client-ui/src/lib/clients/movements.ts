import { useState, useEffect } from 'react';
import logger from '../logger';
import type { Movement } from '../types/movment';

export type MovementsSearchParams = {
  offset: number,
  limit: number
  accountAddress: string | null,
  eventTypes: string[],
  tokens: string[],
}

const useMovementsData = (searchParams: MovementsSearchParams) => {
  const [data, setData] = useState<Movement[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);


  useEffect(() => {
    setLoading(true);

    const fetchData = async () => {
      try {
        const queryParams: Record<string, string> = {
          offset: searchParams.offset.toString(),
          limit: searchParams.limit.toString()
        };
        if (searchParams.accountAddress) queryParams['address'] = searchParams.accountAddress;
        if (searchParams.eventTypes.length > 0) queryParams['event_types'] = searchParams.eventTypes.join(',');
        if (searchParams.tokens.length > 0) queryParams['tokens'] = searchParams.tokens.join(',');
        const query = new URLSearchParams(queryParams).toString();
        logger.info("Fetching with query params", query);
        const response = await fetch(`http://127.0.0.1:8080/movements?${query}`);

        if (!response.ok) {
          if (response.status === 400) {
            setError(await response.text());
          } else {
            setError("Network response was not ok");
          }
        } else {
          const result = await response.json();
          logger.info("Fetched data", result);
          setData(result);
          setError(null);
        }
      } catch (error) {
        logger.error("Error fetching data", error);
        setError("Error fetching data");
      }
      finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [searchParams]);

  return { data, loading, error };
};

export default useMovementsData;
