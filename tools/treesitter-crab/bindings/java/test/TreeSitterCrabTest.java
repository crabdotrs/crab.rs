import io.github.treesitter.jtreesitter.Language;
import io.github.treesitter.jtreesitter.crab.TreeSitterCrab;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;

public class TreeSitterCrabTest {
    @Test
    public void testCanLoadLanguage() {
        assertDoesNotThrow(() -> new Language(TreeSitterCrab.language()));
    }
}
