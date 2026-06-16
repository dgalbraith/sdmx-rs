<details>
<summary>XSD contract: <code>ItemSchemeType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="ItemSchemeType" abstract="true">
		<xs:annotation>
			<xs:documentation>ItemSchemeType is an abstract base type for all item scheme objects. It contains a collection of items. Concrete instances of this type should restrict the actual types of items allowed within the scheme.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="common:MaintainableType">
				<xs:sequence>
					<xs:sequence minOccurs="0" maxOccurs="unbounded">
						<xs:element ref="Item"/>
					</xs:sequence>
				</xs:sequence>
				<xs:attribute name="isPartial" type="xs:boolean" use="optional" default="false">
					<xs:annotation>
						<xs:documentation>The isPartial, if true, indicates that only the relevant portion of the item scheme is being communicated. This is used in cases where a codelist is returned for a data structure in the context of a constraint.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
