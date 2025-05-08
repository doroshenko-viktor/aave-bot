import { Box, Flex, Heading, VStack } from '@chakra-ui/react'
import { useColorModeValue } from './components/ui/color-mode'
import { ButtonControlPanel } from './components/ui/buttons'
import MovementHistory from './components/movements/history'

function App() {

  return (
    <>
      <Flex direction="column" minH="100vh" minW="100vw">
        {/* Header */}
        <Box
          as="header"
          py={4}
          px={6}
          borderBottomWidth="1px"
          borderColor={useColorModeValue('gray.100', 'gray.700')}
        >
          <Heading as="h1" size="xl" textAlign="left">
            AAVE explorer
          </Heading>
        </Box>

        {/* Main Content Area (Flex container for sidebar and main content) */}
        <Flex flex={1} direction={{ base: 'column', md: 'row' }}>
          {/* Control Panel (Sidebar) */}
          <Box
            as="aside"
            w={{ base: '100%', md: '250px' }}
            p={4}
            borderRightWidth={['0px', '0px', '1px']}
            borderColor={useColorModeValue('gray.100', 'gray.700')}
            minHeight={{ md: 'calc(100vh - 64px)' }}
          >
            <VStack align="start"  >
              <ButtonControlPanel>History</ButtonControlPanel>
              <ButtonControlPanel>Settings</ButtonControlPanel>
            </VStack>
          </Box>

          {/* Main Content */}
          <Box as="main" flex={1} p={6}>
            <MovementHistory />
          </Box>
        </Flex>
      </Flex>
    </>
  )
}

export default App
