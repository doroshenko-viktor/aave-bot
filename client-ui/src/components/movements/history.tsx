import {
  Box, Heading, Spinner, Table,
} from "@chakra-ui/react";
import { useMemo, useState } from "react";
import logger from "../../lib/logger";
import type { Movement } from "../../lib/types/movment";
import useMovementsData, { type MovementsSearchParams } from "../../lib/clients/movements";
import MovementsFilterForm from "./filter-form";
import useTokensData from "../../lib/clients/tokens";
import AmountRepr from "../ui/amount-repr";

export default function MovementHistory() {
  const [searchParams, setSearchParams] = useState<MovementsSearchParams>({
    offset: 0,
    limit: 20,
    accountAddress: null,
    eventTypes: [],
    tokens: [],
  })
  const { data, loading, error } = useMovementsData(searchParams);

  return (
    <>
      <MovementsFilterForm initFilters={searchParams} onFilter={(filters) => {
        logger.info("Movements filter changed", filters);
        setSearchParams(filters);
      }} />
      <AaveMovementTablePresentation data={data} error={error} loading={loading} />
    </>
  )

}

type AaveMovementTablePresentationProps = {
  data: Movement[];
  error: string | null;
  loading: boolean;
}

function AaveMovementTablePresentation({ data, error, loading }: AaveMovementTablePresentationProps): React.ReactElement {
  const { tokens, loadingTokens, errorTokens } = useTokensData();
  const tokenMap = useMemo(() => {
    const map = new Map<number, string>();
    tokens?.forEach((token) => {
      map.set(token.id, token.token_symbol);
    });
    return map;
  }, [tokens]);

  if (loading || loadingTokens) {
    return (
      <Box minWidth="100%" minHeight="100vh" alignContent="center" justifyContent="center" display="flex" flexDirection="row" alignItems="center">
        <Spinner />
      </Box>
    )
  }

  if (error || errorTokens) {
    return (
      <Box minWidth="100%">
        <Heading>Error:</Heading>
        <Box>Error loading data</Box>
      </Box>
    )
  }
  return (
    <Table.ScrollArea rounded="md" borderWidth="1px" maxW="100%">
      <Table.Root size="md" interactive variant="line" native >
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeader >Event type</Table.ColumnHeader>
            <Table.ColumnHeader>Account Address</Table.ColumnHeader>
            <Table.ColumnHeader>Token</Table.ColumnHeader>
            <Table.ColumnHeader>Amount</Table.ColumnHeader>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {
            data?.map((movement, ind) => {
              const token = tokenMap.get(movement.token_id) ?? movement.token_id.toString();

              return <AaveMovementRow id={ind} movement={movement} token_symbol={token} key={ind} />
            })}
        </Table.Body>
      </Table.Root>
    </Table.ScrollArea>
  )
}

function AaveMovementRow({ id, movement, token_symbol }: { id: number, movement: Movement, token_symbol: string }): React.ReactElement {
  return (
    <Table.Row key={id}>
      <Table.Cell>{movement.event_type}</Table.Cell>
      <Table.Cell>0x{movement.account_address}</Table.Cell>
      <Table.Cell>{token_symbol}</Table.Cell>
      <Table.Cell> <AmountRepr weiAmount={movement.native_token_amount} /></Table.Cell>
    </Table.Row>
  )
}
