import { useState, useEffect } from 'react';
import logger from '../logger';
import type { Token } from '../types/tokens';


const useTokensData = () => {
  const [tokens, setData] = useState<Token[]>([]);
  const [loadingTokens, setLoading] = useState<boolean>(false);
  const [errorTokens, setError] = useState<string | null>(null);


  useEffect(() => {
    setLoading(true);

    const fetchData = async () => {
      try {
        logger.info("Fetching tokens...");
        const response = await fetch(`http://127.0.0.1:8080/tokens`);

        if (!response.ok) {
          if (response.status === 400) {
            setError(await response.text());
          } else {
            setError("Retrieving tokens network response was not ok");
          }
        } else {
          const result = await response.json();
          logger.info("Fetched data", result);
          setData(result);
          setError(null);
        }
      } catch (error) {
        logger.error("Error fetching tokens", error);
        setError("Error fetching tokens");
      }
      finally {
        setLoading(false);
      }
    };
    fetchData();
  }, []);

  return { tokens, loadingTokens, errorTokens };
};

export default useTokensData;
