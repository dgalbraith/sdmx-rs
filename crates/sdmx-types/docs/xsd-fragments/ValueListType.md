<details>
<summary>XSD contract: <code>ValueListType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="ValueListType">
		<xs:annotation>
			<xs:documentation>ValueListType defines the structure of value list. These represent a closed set of values the can occur for a dimension, measure, or attribute. These may be values, or values with names and descriptions (similar to a codelist).</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="ValueListBaseType">
				<xs:sequence>
					<xs:element name="ValueItem" type="ValueItemType" minOccurs="0" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation></xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
