import { useMemo, useState } from 'react';
import { Input, Select, Button, Stack, Field, Portal, createListCollection, Spinner } from '@chakra-ui/react';
import type { MovementsSearchParams } from '../../lib/clients/movements';
import logger from '../../lib/logger';
import useTokensData from '../../lib/clients/tokens';

const eventTypeCollection = createListCollection({
  items: [
    { value: 'supply', label: 'Supply' },
    { value: 'withdraw', label: 'Withdraw' },
    { value: 'borrow', label: 'Borrow' },
    { value: 'repay', label: 'Repay' },
    { value: 'collateral_transfer', label: 'Collateral transfer' },
    { value: 'liquidation_repay_debt', label: 'Liquidation repay debt' },
    { value: 'liquidation_withdraw_collateral', label: 'Liquidation withdraw collateral' },
  ],
});

type Props = {
  initFilters: MovementsSearchParams,
  onFilter: (filters: MovementsSearchParams) => void;
};

const MovementsFilterForm = ({ initFilters, onFilter }: Props) => {
  const [filters, setFilters] = useState(initFilters);
  const { tokens, loadingTokens } = useTokensData();

  const tokensCollection = useMemo(() => {
    return createListCollection({
      items: tokens ?? [],
      itemToString: (token) => token.token_symbol,
      itemToValue: (token) => token.id.toString(),
    })
  }, [tokens])

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    logger.info('changed event types', e);
    const { name, value } = e.target;
    setFilters((prev) => ({ ...prev, [name]: value }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    logger.info('submit', filters);
    onFilter(filters);
  };

  return (
    <form onSubmit={handleSubmit}>
      <Stack direction={{ base: 'column', md: 'row' }} mb={4}>
        <Field.Root>
          <Field.Label>Account Address</Field.Label>
          <Input
            name="accountAddress"
            value={filters.accountAddress || ''}
            onChange={handleChange}
            placeholder="Address"
          />
        </Field.Root>
        <Field.Root>
          <Select.Root
            multiple
            variant="outline"
            size="md"
            collection={eventTypeCollection}
            name="eventTypes"
            value={filters.eventTypes}
            onValueChange={(e) => {
              logger.info('changed event types', e.value);
              setFilters((prev) => ({ ...prev, eventTypes: e.value }))
            }}
          >
            <Select.HiddenSelect />
            <Select.Label>Event Types</Select.Label>
            <Select.Control>
              <Select.Trigger>
                <Select.ValueText placeholder="Select event types" />
              </Select.Trigger>
              <Select.IndicatorGroup>
                <Select.Indicator />
              </Select.IndicatorGroup>
            </Select.Control>
            <Portal>
              <Select.Positioner>
                <Select.Content>
                  {eventTypeCollection.items.map((eventType) => (
                    <Select.Item item={eventType} key={eventType.value}>
                      {eventType.label}
                      <Select.ItemIndicator />
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Positioner>
            </Portal>
          </Select.Root>
        </Field.Root>

        {/* Tokens */}
        <Field.Root>
          <Select.Root
            multiple
            variant="outline"
            size="md"
            collection={tokensCollection}
            name="tokens"
            value={filters.tokens}
            onValueChange={(e) => {
              logger.info('changed tokens', e.value);
              setFilters((prev) => ({ ...prev, tokens: e.value }))
            }}
          >
            <Select.HiddenSelect />
            <Select.Label>Tokens</Select.Label>
            <Select.Control>
              <Select.Trigger>
                <Select.ValueText placeholder="Select tokens" />
              </Select.Trigger>
              <Select.IndicatorGroup>
                {loadingTokens && (
                  <Spinner size="xs" borderWidth="1.5px" color="fg.muted" />
                )}
                <Select.Indicator />
              </Select.IndicatorGroup>
            </Select.Control>
            <Portal>
              <Select.Positioner>
                <Select.Content>
                  {tokensCollection.items.map((token) => (
                    <Select.Item item={token} key={token.id}>
                      {token.token_symbol}
                      <Select.ItemIndicator />
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Positioner>
            </Portal>
          </Select.Root>
        </Field.Root>
        <Button variant="subtle" alignSelf="flex-end" onClick={() => setFilters(initFilters)}>
          Clear
        </Button>
        <Button type="submit" alignSelf="flex-end">
          Filter
        </Button>
      </Stack>
    </form >
  );
};
export default MovementsFilterForm;
